use std::rc::Rc;

use super::{Instruction, Progression};
use crate::{
    native::NativeFunction,
    object::{
        numeric::{FloatingType, IntegralType},
        Object, WrappedClassObject, RuntimeValue,
    },
    Context, VM, arg, pop
};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use parse::{
    attributes::CodeAttribute,
    classfile::{Method, Resolvable},
    flags::MethodAccessFlag,
    pool::ConstantEntry,
};
use support::descriptor::MethodType;
use tracing::info;

#[derive(Debug)]
pub struct InvokeVirtual {
    pub(crate) index: u16,
}

impl Instruction for InvokeVirtual {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let cls = ctx.class.read();
        let pool = cls.constant_pool();

        // The unsigned indexbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        drop(cls);

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name) = match pool_entry {
            ConstantEntry::Method(data) => {
                let name_and_type = data.name_and_type.resolve();
                let method_name = name_and_type.name.resolve().try_string()?;

                let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
                let method_descriptor = MethodType::parse(method_descriptor)?;

                let class = data.class.resolve();
                let class = class.name.resolve().try_string()?;

                (method_name, method_descriptor, class)
            }
            ConstantEntry::InterfaceMethod(data) => {
                let name_and_type = data.name_and_type.resolve();
                let method_name = name_and_type.name.resolve().try_string()?;

                let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
                let method_descriptor = MethodType::parse(method_descriptor)?;

                let class = data.class.resolve();
                let class = class.name.resolve().try_string()?;

                (method_name, method_descriptor, class)
            }
            e => return Err(anyhow!("expected interface method / method, got {:#?}", e)),
        };

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.load_class(class_name.clone())?;
        let loaded_method = resolve_method(
            vm,
            Rc::clone(&loaded_class),
            method_name.clone(),
            method_descriptor.to_string(),
        )?
        .ok_or(anyhow!(
            "no method {} {} in {} or its parents",
            method_name,
            method_descriptor.to_string(),
            class_name
        ))?;

        // If the resolved method is not signature polymorphic (§2.9.3), then
        // the invokevirtual instruction proceeds as follows.
        // TODO: Support signature polymorphic methods

        // NOTE: We must get the args before relative resolution.
        // This is because the `objectref` lives at the "bottom" of the stack,
        // below all of the args.
        let mut reversed_descriptor = method_descriptor.clone();
        reversed_descriptor.parameters.reverse();
        let mut args_for_call = Vec::new();
        for _arg in reversed_descriptor.parameters.iter() {
            // TODO: Validate against FieldType in descriptor
            let arg = ctx.operands.pop().ok_or(anyhow!("not enough args"))?;
            if let Some(int) = arg.as_integral() {
                if int.ty == IntegralType::Long {
                    args_for_call.push(arg.clone());
                }
            }

            if let Some(float) = arg.as_floating() {
                if float.ty == FloatingType::Double {
                    args_for_call.push(arg.clone());
                }
            }
            args_for_call.push(arg.clone());
        }

        let objectref = arg!(ctx, "objectref" => Object);

        // Let C be the class of objectref. A method is selected with respect
        // to C and the resolved method (§5.4.6). This is the method to be invoked.
        let objectclass = objectref
            .read()
            .class()
            .context("objecref had no class")?;

        let (selected_class, selected_method) =
            select_method(vm, objectclass, loaded_class, loaded_method)?.ok_or(anyhow!(
                "could not resolve method {} {}",
                method_name,
                method_descriptor.to_string()
            ))?;

        args_for_call.push(RuntimeValue::Object(objectref));
        args_for_call.reverse();

        // If the method to be invoked is native and the platform-dependent
        // code that implements it has not yet been bound (§5.6) into
        // the Java Virtual Machine, that is done. The nargs argument
        // values and objectref are popped from the operand stack and are
        // passed as parameters to the code that implements the method.
        // The parameters are passed and the code is invoked in an
        // implementation-dependent manner.
        let exec_result = if !selected_method.flags.has(MethodAccessFlag::NATIVE) {
            // Must load the context if and only if the method is not native.
            // Native methods do not have a code attribute.
            let code = selected_method
                .attributes
                .known_attribute::<CodeAttribute>(selected_class.read().constant_pool())?;

            let new_context = Context {
                code,
                class: Rc::clone(&selected_class),
                pc: 0,
                operands: vec![],
                locals: args_for_call,
            };

            // The new frame is then made current, and the Java Virtual Machine pc is set
            // to the opcode of the first instruction of the method to be invoked.
            // Execution continues with the first instruction of the method.
            vm.run(new_context)
        } else {
            let lookup = selected_class
                .read()
                .fetch_native((method_name.clone(), method_descriptor.to_string()))
                .ok_or(anyhow!(
                    "no native method {} {:?} {} / {}",
                    class_name,
                    selected_method.flags.flags,
                    method_name,
                    method_descriptor.to_string()
                ))?;

            match lookup {
                NativeFunction::Static(func) => func(Rc::clone(&selected_class), args_for_call, vm),
                _ => {
                    return Err(anyhow!(
                        "attempted to InvokeStatic an instance native method"
                    ))
                }
            }
        };

        if let Err(e) = exec_result {
            return Err(e.context(format!(
                "when interpreting {} in {}",
                method_name, class_name
            )));
        }

        // Caller gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result.unwrap() {
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
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let cls = ctx.class.read();
        let pool = cls.constant_pool();

        // The unsigned indexbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        drop(cls);

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name) = match pool_entry {
            ConstantEntry::Method(data) => {
                let name_and_type = data.name_and_type.resolve();
                let method_name = name_and_type.name.resolve().try_string()?;

                let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
                let method_descriptor = MethodType::parse(method_descriptor)?;

                let class = data.class.resolve();
                let class = class.name.resolve().try_string()?;

                (method_name, method_descriptor, class)
            }
            ConstantEntry::InterfaceMethod(data) => {
                let name_and_type = data.name_and_type.resolve();
                let method_name = name_and_type.name.resolve().try_string()?;

                let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
                let method_descriptor = MethodType::parse(method_descriptor)?;

                let class = data.class.resolve();
                let class = class.name.resolve().try_string()?;

                (method_name, method_descriptor, class)
            }
            e => return Err(anyhow!("expected interface method / method, got {:#?}", e)),
        };

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.load_class(class_name.clone())?;
        let loaded_method = loaded_class
            .read()
            .get_method((method_name.clone(), method_descriptor.to_string()))
            .context(format!("no method {} in {}", method_name, class_name))?;

        // The resolved method must not be an instance initialization method,
        // or the class or interface initialization method (§2.9.1, §2.9.2).
        if method_name == "<clinit>" || method_name == "<init>" {
            return Err(anyhow!(
                "cannot call static method {}, it is a class initialisation method",
                method_name
            ));
        }

        // On successful resolution of the method, the class or interface that
        // declared the resolved method is initialized if that class or interface
        // has not already been initialized (§5.5).
        vm.initialise_class(Rc::clone(&loaded_class))?;

        // If the method is not native, the nargs argument values are popped
        // from the operand stack. A new frame is created on the Java Virtual
        // Machine stack for the method being invoked. The nargs argument
        // values are consecutively made the values of local variables of the
        // new frame, with arg1 in local variable 0 (or, if arg1 is of type
        // long or double, in local variables 0 and 1) and so on.
        let mut reversed_descriptor = method_descriptor.clone();
        reversed_descriptor.parameters.reverse();
        let mut args_for_call = Vec::new();
        for _arg in reversed_descriptor.parameters.iter() {
            // TODO: Validate against FieldType in descriptor
            let arg = ctx.operands.pop().ok_or(anyhow!("not enough args"))?;
            if let Some(int) = arg.as_integral() {
                if int.ty == IntegralType::Long {
                    args_for_call.push(arg.clone());
                }
            }

            if let Some(float) = arg.as_floating() {
                if float.ty == FloatingType::Double {
                    args_for_call.push(arg.clone());
                }
            }
            args_for_call.push(arg.clone());
        }

        args_for_call.reverse();

        let exec_result = if !loaded_method.flags.has(MethodAccessFlag::NATIVE) {
            // Must load the context if and only if the method is not native.
            // Native methods do not have a code attribute.
            let code = loaded_method
                .attributes
                .known_attribute::<CodeAttribute>(loaded_class.read().constant_pool())?;

            let new_context = Context {
                code,
                class: Rc::clone(&loaded_class),
                pc: 0,
                operands: vec![],
                locals: args_for_call,
            };

            // The new frame is then made current, and the Java Virtual Machine pc is set
            // to the opcode of the first instruction of the method to be invoked.
            // Execution continues with the first instruction of the method.
            vm.run(new_context)
        } else {
            let lookup = loaded_class
                .read()
                .fetch_native((method_name.clone(), method_descriptor.to_string()))
                .ok_or(anyhow!(
                    "no native method {} {:?} {} / {}",
                    class_name,
                    loaded_method.flags.flags,
                    method_name,
                    method_descriptor.to_string()
                ))?;

            match lookup {
                NativeFunction::Static(func) => func(Rc::clone(&loaded_class), args_for_call, vm),
                _ => {
                    return Err(anyhow!(
                        "attempted to InvokeStatic an instance native method"
                    ))
                }
            }
        };

        if let Err(e) = exec_result {
            return Err(e.context(format!(
                "when interpreting {} in {}",
                method_name, class_name
            )));
        }

        // Caller gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result.unwrap() {
            ctx.operands.push(return_value);
        }

        Ok(Progression::Next)
    }
}

fn resolve_method(
    vm: &mut VM,
    class: WrappedClassObject,
    method_name: String,
    method_descriptor: String,
) -> Result<Option<Method>> {
    // To resolve an unresolved symbolic reference from D to a method in a class C, the
    // symbolic reference to C given by the method reference is first resolved (§5.4.3.1).

    // When resolving a method reference:
    // 1. If C is an interface, method resolution throws an IncompatibleClassChangeError.
    if class.read().is_interface() {
        return Err(anyhow!("cannot resolve method on interface"));
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
        .read()
        .get_method((method_name.clone(), method_descriptor.clone()));

    if let Some(class_method) = class_method {
        return Ok(Some(class_method));
    }

    // • Otherwise, if C has a superclass, step 2 of method resolution is recursively
    // invoked on the direct superclass of C.
    if let Some(super_class) = class.read().super_class() {
        let class_name = super_class.read().get_class_name().clone();
        let super_class = vm.class_loader.load_class(class_name)?;

        return resolve_method(vm, super_class, method_name, method_descriptor);
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

    Ok(None)
}

fn select_method(
    vm: &mut VM,
    class: WrappedClassObject,
    declared_class: WrappedClassObject,
    method: Method,
) -> Result<Option<(WrappedClassObject, Method)>> {
    // During execution of an invokeinterface or invokevirtual instruction, a method is
    // selected with respect to (i) the run-time type of the object on the stack, and (ii) a
    // method that was previously resolved by the instruction. The rules to select a method
    // with respect to a class or interface C and a method mR are as follows:

    let (method_name, method_descriptor) = (
        method.name.resolve().try_string()?,
        method.descriptor.resolve().try_string()?,
    );

    let class_name = {
        let class = class.read();
        class.get_class_name().clone()
    };

    let declared_class_name = {
        let class = declared_class.read();
        class.get_class_name().clone()
    };

    info!(
        "select_method: {} {} in {} (declared as {})",
        method_name, method_descriptor, class_name, declared_class_name
    );

    // 1. If mR is marked ACC_PRIVATE, then it is the selected method.
    if method.flags.has(MethodAccessFlag::PRIVATE) {
        info!("select_method: was private, using it");

        let method = declared_class
            .read()
            .get_method((method_name, method_descriptor))
            .ok_or(anyhow!("could not resolve method"))?;

        return Ok(Some((declared_class, method)));
    }

    // 2. Otherwise, the selected method is determined by the following lookup procedure:
    // If C contains a declaration of an instance method m that can override mR (§5.4.5), then m is the selected method.
    let instance_method = class
        .read()
        .get_method((method_name.clone(), method_descriptor.clone()));

    if let Some(instance_method) = instance_method {
        info!("select_method: found instance method");
        if method_can_override(&method, &instance_method) {
            info!("select_method: could override, using it");

            return Ok(Some((class, instance_method)));
        }
        info!("select_method: could not override, skipping it");
    }

    // Otherwise, if C has a superclass, a search for a declaration of an instance
    // method that can override mR is performed, starting with the direct superclass
    // of C and continuing with the direct superclass of that class, and so forth, until
    // a method is found or no further superclasses exist. If a method is found, it
    // is the selected method.
    let super_class = class.read().super_class();
    if let Some(super_class) = super_class {
        let class_name = super_class.read().get_class_name().clone();
        info!(
            "select_method: attempting resolution in super class {}",
            class_name
        );
        let super_class = vm.class_loader.load_class(class_name)?;

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
