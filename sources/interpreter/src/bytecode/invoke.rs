use super::{Instruction, Progression};
use crate::{
    arg,
    error::{Frame, RuntimeException, Throwable},
    internal,
    native::NativeFunction,
    object::{
        builtins::{Array, BuiltinString, Class, Object},
        layout::types::{Bool, Byte, Char, Double, Float, Int, Long, Short},
        mem::{FieldRef, HasObjectHeader, RefTo},
        numeric::{FloatingType, IntegralType},
        runtime::RuntimeValue,
    },
    pop, Context, VM,
};
use anyhow::Context as AnyhowContext;

use parse::{
    attributes::CodeAttribute,
    classfile::{Method, Resolvable},
    flags::{FieldAccessFlag, MethodAccessFlag},
    pool::ConstantEntry,
};
use support::{
    descriptor::{BaseType, FieldType, MethodType},
    encoding::{decode_string, CompactEncoding},
};
use tracing::{debug, info};

#[derive(Debug)]
pub struct InvokeVirtual {
    pub(crate) index: u16,
}

impl Instruction for InvokeVirtual {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let cls = ctx.class.clone();
        let cls = cls.borrow();
        let pool = &cls.class_file().constant_pool;

        // The unsigned indExbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name, _) = to_method_info(pool_entry)?;

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.for_name(class_name.clone())?;

        let loaded_method = resolve_class_method(
            vm,
            loaded_class.clone(),
            method_name.clone(),
            method_descriptor.to_string(),
        )?;

        // If the resolved method is not signature polymorphic (§2.9.3), then
        // the invokevirtual instruction proceeds as follows.
        // TODO: Support signature polymorphic methods

        vm.frames.push(Frame {
            method_name: method_name.clone(),
            class_name,
        });

        debug!("Invoking: {:#?}", vm.frames.last());

        // NOTE: We must get the args before relative resolution.
        // This is because the `objectref` lives at the "bottom" of the stack,
        // below all of the args.
        let mut args_for_call = clone_args_from_operands(method_descriptor.clone(), ctx)?;
        let objectref = arg!(ctx, "objectref" => Object);

        // Let C be the class of objectref. A method is selected with respect
        // to C and the resolved method (§5.4.6). This is the method to be invoked.
        let objectclass = objectref.borrow().header().class.clone();

        let (selected_class, selected_method) =
            select_method(vm, objectclass, loaded_class, loaded_method)?.ok_or(internal!(
                "could not resolve method {} {}",
                method_name,
                method_descriptor.to_string()
            ))?;

        args_for_call.push(RuntimeValue::Object(objectref));
        args_for_call.reverse();

        let exec_result = do_call(vm, selected_method, selected_class, args_for_call)?;

        debug!("Returned from: {:#?}", vm.frames.last());
        vm.frames.pop();

        // Callee gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result {
            ctx.operands.push(return_value);
        }

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct InvokeSpecial {
    pub(crate) index: u16,
}

impl Instruction for InvokeSpecial {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let cls = ctx.class.borrow();
        let pool = &cls.class_file().constant_pool;

        // The unsigned indexbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name, _) = to_method_info(pool_entry)?;

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.for_name(class_name.clone())?;
        let loaded_method = resolve_class_method(
            vm,
            loaded_class.clone(),
            method_name.clone(),
            method_descriptor.to_string(),
        )?;

        vm.frames.push(Frame {
            method_name: method_name.clone(),
            class_name,
        });

        debug!("Invoking: {:#?}", vm.frames.last());

        // NOTE: We must get the args before resolution.
        // This is because the `objectref` lives at the "bottom" of the stack,
        // below all of the args.
        let mut args_for_call = clone_args_from_operands(method_descriptor.clone(), ctx)?;

        let objectref = arg!(ctx, "objectref" => Object);

        let (selected_class, selected_method) =
            select_special_method(vm, loaded_class.clone(), loaded_class, loaded_method)?.ok_or(
                internal!(
                    "could not resolve special method {} {}",
                    method_name,
                    method_descriptor.to_string()
                ),
            )?;

        args_for_call.push(RuntimeValue::Object(objectref));
        args_for_call.reverse();

        let exec_result = do_call(vm, selected_method, selected_class, args_for_call)?;

        debug!("Returned from: {:#?}", vm.frames.last());
        vm.frames.pop();

        // Callee gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result {
            ctx.operands.push(return_value);
        }

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct InvokeStatic {
    pub(crate) index: u16,
}

impl Instruction for InvokeStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let cls = ctx.class.borrow();
        let pool = &cls.class_file().constant_pool;

        // The unsigned indexbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name, location) = to_method_info(pool_entry)?;

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.for_name(class_name.clone())?;
        let loaded_method = match location {
            MethodLocation::Interface => resolve_interface_method(
                vm,
                loaded_class.clone(),
                method_name.clone(),
                method_descriptor.to_string(),
            ),
            MethodLocation::Class => resolve_class_method(
                vm,
                loaded_class.clone(),
                method_name.clone(),
                method_descriptor.to_string(),
            ),
        }?;

        // The resolved method must not be an instance initialization method,
        // or the class or interface initialization method (§2.9.1, §2.9.2).
        if method_name == "<clinit>" || method_name == "<init>" {
            return Err(internal!(
                "cannot call static method {}, it is a class initialisation method",
                method_name
            ));
        }

        // On successful resolution of the method, the class or interface that
        // declared the resolved method is initialized if that class or interface
        // has not already been initialized (§5.5).
        vm.initialise_class(loaded_class.clone())?;

        // If the method is not native, the nargs argument values are popped
        // from the operand stack. A new frame is created on the Java Virtual
        // Machine stack for the method being invoked. The nargs argument
        // values are consecutively made the values of local variables of the
        // new frame, with arg1 in local variable 0 (or, if arg1 is of type
        // long or double, in local variables 0 and 1) and so on.

        vm.frames.push(Frame {
            method_name,
            class_name,
        });

        debug!("Invoking: {:#?}", vm.frames.last());

        let mut args_for_call = clone_args_from_operands(method_descriptor, ctx)?;
        args_for_call.reverse();

        let exec_result = do_call(vm, loaded_method, loaded_class, args_for_call)?;

        debug!("Returned from: {:#?}", vm.frames.last());
        vm.frames.pop();

        // Callee gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result {
            ctx.operands.push(return_value);
        }

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct InvokeInterface {
    pub(crate) index: u16,
    pub(crate) count: u8,
    pub(crate) zero: u8,
}

impl Instruction for InvokeInterface {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The value of the fourth operand byte must always be zero.
        if self.zero != 0 {
            return Err(internal!("zero was not zero"));
        }

        // The count operand is an unsigned byte that must not be zero
        if self.count == 0 {
            return Err(internal!("count was 0"));
        }

        let cls = ctx.class.borrow();
        let pool = &cls.class_file().constant_pool;

        // The unsigned indexbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name, _) = to_method_info(pool_entry)?;

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.for_name(class_name.clone())?;
        let loaded_method = resolve_interface_method(
            vm,
            loaded_class.clone(),
            method_name.clone(),
            method_descriptor.to_string(),
        )?;

        // The resolved method must not be an instance initialization method,
        // or the class or interface initialization method (§2.9.1, §2.9.2).
        if method_name == "<clinit>" || method_name == "<init>" {
            return Err(internal!(
                "cannot call interface method {}, it is a class initialisation method",
                method_name
            ));
        }

        // NOTE: We must get the args before relative resolution.
        // This is because the `objectref` lives at the "bottom" of the stack,
        // below all of the args.
        let mut args_for_call = clone_args_from_operands(method_descriptor.clone(), ctx)?;
        let objectref = arg!(ctx, "objectref" => Object);

        // Let C be the class of objectref. A method is selected with respect
        // to C and the resolved method (§5.4.6). This is the method to be invoked.
        let objectclass = objectref.borrow().header().class.clone();

        let (selected_class, selected_method) =
            select_method(vm, objectclass, loaded_class, loaded_method)?.ok_or(internal!(
                "could not resolve method {} {}",
                method_name,
                method_descriptor.to_string()
            ))?;

        vm.frames.push(Frame {
            method_name,
            class_name,
        });

        debug!("Invoking: {:#?}", vm.frames.last());
        args_for_call.push(RuntimeValue::Object(objectref));
        args_for_call.reverse();

        let exec_result = do_call(vm, selected_method, selected_class, args_for_call)?;

        debug!("Returned from: {:#?}", vm.frames.last());
        vm.frames.pop();

        // Callee gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result {
            ctx.operands.push(return_value);
        }

        Ok(Progression::Next)
    }
}

fn resolve_class_method(
    vm: &mut VM,
    class: RefTo<Class>,
    method_name: String,
    method_descriptor: String,
) -> Result<Method, Throwable> {
    // To resolve an unresolved symbolic reference from D to a method in a class C, the
    // symbolic reference to C given by the method reference is first resolved (§5.4.3.1).

    // When resolving a method reference:
    // 1. If C is an interface, method resolution throws an IncompatibleClassChangeError.
    if class.borrow().is_interface() {
        return Err(internal!("cannot resolve method on interface"));
    }

    // 2. Otherwise, method resolution attempts to locate the referenced method in C and its superclasses:
    // • If C declares exactly one method with the name specified by the method
    // reference, and the declaration is a signature polymorphic method (§2.9.3),
    // then method lookup succeeds. All the class names mentioned in the
    // descriptor are resolved (§5.4.3.1).
    // The resolved method is the signature polymorphic method declaration. It is
    // not necessary for C to declare a method with the descriptor specified by the
    // method reference.
    // • Otherwise, if C declares a method with the name and descriptor specified by
    // the method reference, method lookup succeeds.

    let class_method = class
        .borrow()
        .class_file()
        .methods
        .locate(method_name.clone(), method_descriptor.clone())
        .cloned();

    if let Some(class_method) = class_method {
        return Ok(class_method.clone());
    }

    // • Otherwise, if C has a superclass, step 2 of method resolution is recursively
    // invoked on the direct superclass of C.
    if let Some(super_class) = class.borrow().super_class().into_option() {
        let class_name = super_class.name();
        let super_class = vm.class_loader.for_name(class_name.to_string())?;

        return resolve_class_method(vm, super_class, method_name, method_descriptor);
    }

    // Otherwise, method resolution attempts to locate the referenced method in the
    // superinterfaces of the specified class C:
    // • If the maximally-specific superinterface methods of C for the name and
    // descriptor specified by the method reference include exactly one method that
    // does not have its ACC_ABSTRACT flag set, then this method is chosen and
    // method lookup succeeds.
    // • Otherwise, if any superinterface of C declares a method with the name and
    // descriptor specified by the method reference that has neither its ACC_PRIVATE
    // flag nor its ACC_STATIC flag set, one of these is arbitrarily chosen and method
    // lookup succeeds.

    // TODO: Interface resolution

    // • Otherwise, method lookup fails.

    Err(internal!(
        "could not resolve method {} ({}) in {}",
        method_name,
        method_descriptor,
        class.borrow().name()
    ))
}

fn resolve_interface_method(
    vm: &mut VM,
    class: RefTo<Class>,
    method_name: String,
    method_descriptor: String,
) -> Result<Method, Throwable> {
    // To resolve an unresolved symbolic reference from D to an interface method in an
    // interface C, the symbolic reference to C given by the interface method reference is first resolved (§5.4.3.1)

    // When resolving an interface method reference:
    // 1. If C is not an interface, interface method resolution throws an IncompatibleClassChangeError
    if !class.borrow().is_interface() {
        return Err(internal!("cannot resolve interface method on class"));
    }

    // 2. Otherwise, if C declares a method with the name and descriptor specified by
    // the interface method reference, method lookup succeeds.
    let own_method = class
        .borrow()
        .class_file()
        .methods
        .locate(method_name.clone(), method_descriptor.clone())
        .cloned();

    if let Some(own_method) = own_method {
        return Ok(own_method.clone());
    }

    // 3. Otherwise, if the class Object declares a method with the name and descriptor
    // specified by the interface method reference, which has its ACC_PUBLIC flag set
    // and does not have its ACC_STATIC flag set, method lookup succeeds.
    // TODO: Respect the flags
    if let Some(super_class) = class.borrow().super_class().into_option() {
        let class_name = super_class.name();
        let super_class = vm.class_loader.for_name(class_name.to_string())?;

        return resolve_interface_method(vm, super_class, method_name, method_descriptor);
    }

    // 4. Otherwise, if the maximally-specific superinterface methods (§5.4.3.3) of C
    // for the name and descriptor specified by the method reference include exactly
    // one method that does not have its ACC_ABSTRACT flag set, then this method is
    // chosen and method lookup succeeds

    // TODO: Interface resolution

    // 5. Otherwise, if any superinterface of C declares a method with the name and
    // descriptor specified by the method reference that has neither its ACC_PRIVATE
    // flag nor its ACC_STATIC flag set, one of these is arbitrarily chosen and method
    // lookup succeeds

    // • Otherwise, method lookup fails.

    Err(internal!(
        "could not resolve method {} ({}) in {}",
        method_name,
        method_descriptor,
        class.borrow().name()
    ))
}

fn select_special_method(
    vm: &mut VM,
    class: RefTo<Class>,
    declared_class: RefTo<Class>,
    method: Method,
) -> Result<Option<(RefTo<Class>, Method)>, Throwable> {
    let (method_name, method_descriptor) = (
        method.name.resolve().try_string()?,
        method.descriptor.resolve().try_string()?,
    );

    // Let C be the class or interface named by the symbolic reference.

    // 1. If C contains a declaration for an instance method with the
    // same name and descriptor as the resolved method, then it is
    // the method to be invoked.
    let instance_method = class
        .borrow()
        .class_file()
        .methods
        .locate(method_name.clone(), method_descriptor.clone())
        .cloned();

    if let Some(instance_method) = instance_method {
        return Ok(Some((class, instance_method.clone())));
    }

    // 2. Otherwise, if C is a class and has a superclass, a search for
    // a declaration of an instance method with the same name
    // and descriptor as the resolved method is performed, starting
    // with the direct superclass of C and continuing with the direct
    // superclass of that class, and so forth, until a match is found or
    // no further superclasses exist. If a match is found, then it is the
    // method to be invoked.
    if let Some(super_class) = class.borrow().super_class().into_option() {
        let class_name = super_class.name();
        let _super_class = vm.class_loader.for_name(class_name.to_string())?;

        let super_class = vm.class_loader.for_name(class_name.to_string())?;
        return select_method(vm, super_class, declared_class, method);
    }

    // 3. Otherwise, if C is an interface and the class Object contains a
    // declaration of a public instance method with the same name
    // and descriptor as the resolved method, then it is the method
    // to be invoked.
    // TODO: Implement

    // 4. Otherwise, if there is exactly one maximally-specific method
    // (§5.4.3.3) in the superinterfaces of C that matches the resolved
    // method's name and descriptor and is not abstract, then it is
    // the method to be invoked.
    // TODO: Implement

    Ok(None)
}

fn select_method(
    vm: &mut VM,
    class: RefTo<Class>,
    declared_class: RefTo<Class>,
    method: Method,
) -> Result<Option<(RefTo<Class>, Method)>, Throwable> {
    // During execution of an invokeinterface or invokevirtual instruction, a method is
    // selected with respect to (i) the run-time type of the object on the stack, and (ii) a
    // method that was previously resolved by the instruction. The rules to select a method
    // with respect to a class or interface C and a method mR are as follows:

    let (method_name, method_descriptor) = (
        method.name.resolve().try_string()?,
        method.descriptor.resolve().try_string()?,
    );

    // 1. If mR is marked ACC_PRIVATE, then it is the selected method.
    if method.flags.has(MethodAccessFlag::PRIVATE) {
        let method = declared_class
            .borrow()
            .class_file()
            .methods
            .locate(method_name, method_descriptor)
            .cloned()
            .ok_or(internal!("could not resolve method"))?;

        return Ok(Some((declared_class, method.clone())));
    }

    // 2. Otherwise, the selected method is determined by the following lookup procedure:
    // If C contains a declaration of an instance method m that can override mR (§5.4.5), then m is the selected method.
    let instance_method = class
        .borrow()
        .class_file()
        .methods
        .locate(method_name.clone(), method_descriptor.clone())
        .cloned();

    if let Some(instance_method) = instance_method {
        if method_can_override(&method, &instance_method) {
            return Ok(Some((class, instance_method.clone())));
        }
    }

    // Otherwise, if C has a superclass, a search for a declaration of an instance
    // method that can override mR is performed, starting with the direct superclass
    // of C and continuing with the direct superclass of that class, and so forth, until
    // a method is found or no further superclasses exist. If a method is found, it
    // is the selected method.
    if let Some(super_class) = class.borrow().super_class().into_option() {
        let class_name = super_class.name();
        let _super_class = vm.class_loader.for_name(class_name.to_string())?;

        let super_class = vm.class_loader.for_name(class_name.to_string())?;
        return select_method(vm, super_class, declared_class, method);
    }

    // Otherwise, the maximally-specific superinterface methods of C are
    // determined (§5.4.3.3). If exactly one matches mR's name and descriptor and
    // is not abstract, then it is the selected method.
    // TODO: This, once we figure out a better way to handle super classes

    Ok(None)
}

fn method_can_override(base: &Method, derived: &Method) -> bool {
    // An instance method mC can override another instance method mA iff all of the following are true:
    let (base_name, base_descriptor) = (
        base.name.resolve().string(),
        base.descriptor.resolve().string(),
    );

    let (derived_name, derived_descriptor) = (
        derived.name.resolve().string(),
        derived.descriptor.resolve().string(),
    );

    // mC has the same name and descriptor as mA.
    if base_name != derived_name && base_descriptor != derived_descriptor {
        return false;
    }

    // mC is not marked ACC_PRIVATE.
    let flags = &base.flags;
    if flags.has(MethodAccessFlag::PRIVATE) {
        return false;
    }

    // One of the following is true:
    // – mA is marked ACC_PUBLIC.
    // – mA is marked ACC_PROTECTED.
    // – mA is marked neither ACC_PUBLIC nor ACC_PROTECTED nor ACC_PRIVATE, and
    // either (a) the declaration of mA appears in the same run-time package as the
    // declaration of mC, or (b) if mA is declared in a class A and mC is declared in a class
    // C, then there exists a method mB declared in a class B such that C is a subclass
    // of B and B is a subclass of A and mC can override mB and mB can override mA.

    if flags.has(MethodAccessFlag::PUBLIC) || flags.has(MethodAccessFlag::PROTECTED) {
        return true;
    }

    // TODO: Try resolving package information before returning true
    true
}

#[derive(Debug)]
pub struct New {
    pub(crate) index: u16,
}

impl Instruction for New {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a class or interface type. The named class or interface
        // type is resolved (§5.4.3.1) and should result in a class type.
        let entry: ConstantEntry = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .resolve();

        let object_ty = match entry {
            ConstantEntry::Class(data) => {
                let class_name = data.name.resolve().string();
                vm.class_loader.for_name(class_name)?
            }
            e => return Err(internal!("{:#?} cannot be used to create a new object", e)),
        };

        // Memory for a new instance of that class is allocated from the
        // garbage-collected heap, and the instance variables of the new
        // object are initialized to their default initial values (§2.3, §2.4).

        let layout = object_ty.borrow().instance_layout().clone();
        let ptr = layout.alloc();

        // NOTE: layout.fields() contains the all inherited fields too.
        // FIXME: Do we need to do this if the memory is zero from alloc time?
        for field in layout.fields().values() {
            if field.data.flags.has(FieldAccessFlag::STATIC) {
                continue;
            }

            let field_start = unsafe { ptr.byte_add(field.location.offset) };
            let descriptor = field.data.descriptor.resolve().string();
            let descriptor = FieldType::parse(descriptor)?;
            match descriptor {
                FieldType::Base(ty) => match ty {
                    BaseType::Boolean => unsafe {
                        field_start.cast::<Bool>().write(0);
                    },
                    BaseType::Char => unsafe {
                        field_start.cast::<Char>().write(0);
                    },
                    BaseType::Float => unsafe {
                        field_start.cast::<Float>().write(0.0);
                    },
                    BaseType::Double => unsafe {
                        field_start.cast::<Double>().write(0.0);
                    },
                    BaseType::Byte => unsafe {
                        field_start.cast::<Byte>().write(0);
                    },
                    BaseType::Short => unsafe {
                        field_start.cast::<Short>().write(0);
                    },
                    BaseType::Int => unsafe {
                        field_start.cast::<Int>().write(0);
                    },
                    BaseType::Long => unsafe {
                        field_start.cast::<Long>().write(0);
                    },
                    BaseType::Void => panic!("cannot write default for void"),
                },
                FieldType::Object(_) => {
                    unsafe { field_start.cast::<RefTo<Object>>().write(RefTo::null()) };
                }
                FieldType::Array(_) => {
                    unsafe { field_start.cast::<RefTo<Array<()>>>().write(RefTo::null()) };
                }
            }
        }

        let super_class = object_ty.borrow().super_class();

        unsafe {
            (*ptr).class = object_ty.clone();
            (*ptr).ref_count = 0;
            (*ptr).super_class = super_class;
        }

        // The objectref, a reference to the instance, is pushed onto the operand stack.

        // Safety: We know ptr is not null here
        let objectref = unsafe { RefTo::from_ptr(ptr) };
        ctx.operands.push(RuntimeValue::Object(objectref));

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
enum MethodLocation {
    Interface,
    Class,
}

fn to_method_info(
    pool_entry: ConstantEntry,
) -> Result<(String, MethodType, String, MethodLocation), Throwable> {
    match pool_entry {
        ConstantEntry::Method(data) => {
            let name_and_type = data.name_and_type.resolve();
            let method_name = name_and_type.name.resolve().try_string()?;

            let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
            let method_descriptor = MethodType::parse(method_descriptor)?;

            let class = data.class.resolve();
            let class = class.name.resolve().try_string()?;

            Ok((method_name, method_descriptor, class, MethodLocation::Class))
        }
        ConstantEntry::InterfaceMethod(data) => {
            let name_and_type = data.name_and_type.resolve();
            let method_name = name_and_type.name.resolve().try_string()?;

            let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
            let method_descriptor = MethodType::parse(method_descriptor)?;

            let class = data.class.resolve();
            let class = class.name.resolve().try_string()?;

            Ok((
                method_name,
                method_descriptor,
                class,
                MethodLocation::Interface,
            ))
        }
        e => Err(internal!("expected interface method / method, got {:#?}", e)),
    }
}

fn clone_args_from_operands(
    descriptor: MethodType,
    ctx: &mut Context,
) -> Result<Vec<RuntimeValue>, Throwable>{
    let mut reversed_descriptor = descriptor.clone();
    reversed_descriptor.parameters.reverse();
    let mut args = Vec::new();

    for _arg in reversed_descriptor.parameters.iter() {
        let popped_arg = ctx.operands.pop().ok_or(internal!("not enough args"))?;

        if let Some(int) = popped_arg.as_integral() {
            if int.ty == IntegralType::Long {
                args.push(popped_arg.clone());
            }
        }

        if let Some(float) = popped_arg.as_floating() {
            if float.ty == FloatingType::Double {
                args.push(popped_arg.clone());
            }
        }

        // TODO: Type check args properly

        args.push(popped_arg.clone());
    }

    Ok(args)
}

fn do_call(
    vm: &mut VM,
    method: Method,
    class: RefTo<Class>,
    args: Vec<RuntimeValue>,
) -> Result<Option<RuntimeValue>, Throwable> {
    if !method.flags.has(MethodAccessFlag::NATIVE) {
        // Must load the context if and only if the method is not native.
        // Native methods do not have a code attribute.
        let code = method
            .attributes
            .known_attribute::<CodeAttribute>(&class.borrow().class_file().constant_pool)?;

        let new_context = Context {
            code: code.clone(),
            class: class.clone(),
            pc: 0,
            operands: vec![],
            locals: args.clone(),
        };

        // The new frame is then made current, and the Java Virtual Machine pc is set
        // to the opcode of the first instruction of the method to be invoked.
        // Execution continues with the first instruction of the method.
        let res = vm.run(new_context);

        // See if this code threw a (runtime // java) exception
        // If so, see if we (the caller) can handle it
        if let Err((Throwable::Runtime(err), state)) = &res {
            let ty = err.ty.borrow();

            for entry in &code.exception_table {
                let entry_ty = {
                    let name = entry.catch_type.resolve().name.resolve().string();
                    vm.class_loader.for_name(name)
                }?;

                // The handler supports the type of the exception
                let has_type_match = entry_ty.borrow().is_assignable_to(ty);

                // The handler covers the range of code we just called
                let has_range_match = (entry.start_pc..entry.end_pc).contains(&(state.pc as u16));

                if has_type_match && has_range_match {
                    // We matched, jump to the handler
                    let re_enter_context = Context {
                        code: code.clone(),
                        class,
                        pc: entry.handler_pc as i32,
                        // Push the exception object as the first operand
                        operands: vec![err.obj.clone()],
                        locals: args.clone(),
                    };

                    info!("Re-entering at {}", re_enter_context.pc);

                    return vm.run(re_enter_context).map_err(|e| e.0);
                }
            }
        }

        // We couldn't handle the exception, throw it up
        res.map_err(|e| e.0)
    } else {
        let method_name = method.name.resolve().string();
        let method_descriptor = method.descriptor.resolve().string();

        let lookup = class
            .borrow()
            .native_methods()
            .get(&(method_name.clone(), method_descriptor.clone()))
            .ok_or(internal!(
                "no native method {} {:?} {} / {}",
                class.borrow().name(),
                method.flags.flags,
                method_name,
                method_descriptor
            ))?;

        match lookup {
            NativeFunction::Static(func) => func(class.clone(), args, vm),
            NativeFunction::Instance(func) => {
                let this_ref = args.get(0).unwrap().as_object().unwrap().clone();
                func(this_ref, args, vm)
            }
        }
    }
}

#[derive(Debug)]
pub struct Athrow;

impl Instruction for Athrow {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let _throwable = pop!(ctx);
        let throwable = _throwable
            .as_object()
            .context("throwable was not an object")?
            .clone();

        let throwable = throwable.borrow_mut();
        let class = throwable.header().class.clone();
        let class_name = class.borrow().name();

        let message: FieldRef<RefTo<Object>> = throwable
            .field((
                "detailMessage".to_string(),
                "Ljava/lang/String;".to_string(),
            ))
            .unwrap();

        let message = message.borrow();
        // assert_eq!(message.borrow().ty, ObjectType::String);
        // Safety: Pray
        let message = unsafe { message.cast::<BuiltinString>() };

        let message = message
            .into_option()
            .map(|m| {
                let bytes = m.value.borrow();
                let bytes = bytes.slice().to_vec();

                decode_string((CompactEncoding::from_coder(m.coder), bytes))
            })
            .unwrap_or(Ok("".to_string()))?;

        Ok(Progression::Throw(Throwable::Runtime(RuntimeException {
            message: format!("{}: {}", class_name, message),
            ty: class,
            obj: _throwable,
            sources: vm.frames.clone(),
        })))
    }
}

#[derive(Debug)]
pub struct TableSwitch {
    pub default: i32,
    pub low: i32,
    pub high: i32,
    pub table: Vec<i32>,
}

impl Instruction for TableSwitch {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The index must be of type int and is popped from the operand stack.
        let index = arg!(ctx, "index" => i32);
        let index = index.value as i32;

        let target_address = if index > self.high || index < self.low {
            // If index is less than low or index is greater than high, then
            // a target address is calculated by adding default to the address of
            // the opcode of this tableswitch instruction.
            ctx.pc + self.default
        } else {
            // Otherwise, the offset at position index - low of the jump table is extracted.
            let table_index = index - self.low;

            // The target address is calculated by adding that offset to the address of the
            // opcode of this tableswitch instruction.
            ctx.pc + self.table[table_index as usize]
        };

        // Execution then continues at the target address.
        // NOTE: We could use JumpRel and not do the manual ctx.pc addition above,
        //       but it feels easier to handle this complex jumping logic here and
        //       not get the caller involved in address calculation.
        Ok(Progression::JumpAbs(target_address))
    }
}

#[derive(Debug)]
pub struct LookupSwitch {
    pub default: i32,
    pub pairs: Vec<(i32, i32)>,
}

impl Instruction for LookupSwitch {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The key must be of type int and is popped from the operand stack.
        let key = arg!(ctx, "key" => i32);
        let key = key.value as i32;

        // The key is compared against the match values.
        let found = self.pairs.iter().find(|p| p.0 == key).copied();

        let target_address: i32;

        if let Some(found) = found {
            // If it is equal to one of them, then a target address is calculated by adding
            // the corresponding offset to the address of the opcode of this lookupswitch instruction
            target_address = ctx.pc + found.1;
        } else {
            // If the key does not match any of the
            // match values, the target address is calculated by adding default
            // to the address of the opcode of this lookupswitch instruction.
            target_address = ctx.pc + self.default;
        }

        // Execution then continues at the target address.
        // NOTE: We could use JumpRel and not do the manual ctx.pc addition above,
        //       but it feels easier to handle this complex jumping logic here and
        //       not get the caller involved in address calculation.
        Ok(Progression::JumpAbs(target_address))
    }
}
