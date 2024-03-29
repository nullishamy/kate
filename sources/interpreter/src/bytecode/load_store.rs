use crate::{
    arg,
    error::Throwable,
    internal,
    object::{
        builtins::Object,
        interner::intern_string,
        layout::types::{Bool, Byte, Char, Double, Float, Int, Long, Short},
        mem::{FieldRef, RefTo},
        runtime::{ComputationalType, RuntimeValue},
    },
    pop, Context, VM,
};
use anyhow::Context as AnyhowContext;
use parse::{
    classfile::Resolvable,
    pool::{ConstantEntry, ConstantField},
};
use support::descriptor::{BaseType, FieldType};


use super::{Instruction, Progression};

#[derive(Debug)]
pub struct PushConst {
    pub(crate) value: RuntimeValue,
}

impl Instruction for PushConst {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        ctx.operands.push(self.value.clone());
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Ldc2W {
    pub(crate) index: u16,
}

impl Instruction for Ldc2W {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .try_resolve()
            .context(format!("no value @ index {}", self.index))?;

        match value {
            ConstantEntry::Long(data) => {
                ctx.operands
                    .push(RuntimeValue::Integral((data.bytes as i64).into()));
            }
            ConstantEntry::Double(data) => {
                ctx.operands.push(RuntimeValue::Floating(data.bytes.into()));
            }
            v => return Err(internal!("cannot load {:#?} with ldc2w", v)),
        };

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Ldc {
    pub(crate) index: u16,
}

impl Instruction for Ldc {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .try_resolve()
            .context(format!("no value @ index {}", self.index))?;

        match value {
            ConstantEntry::Integer(data) => {
                ctx.operands
                    .push(RuntimeValue::Integral((data.bytes as i32).into()));
            }
            ConstantEntry::Float(data) => {
                ctx.operands.push(RuntimeValue::Floating(data.bytes.into()));
            }
            ConstantEntry::String(data) => {
                let str = data.string();
                let obj = intern_string(str)?;

                ctx.operands.push(RuntimeValue::Object(obj.erase()))
            }
            ConstantEntry::Class(data) => {
                let class_name = data.name.resolve().string();
                let class = vm.class_loader.for_name(class_name)?;

                ctx.operands.push(RuntimeValue::Object(class.erase()));
            }
            v => return Err(internal!("cannot load {:#?} with ldc", v)),
        };

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct LoadLocal {
    pub(crate) index: usize,
}

impl Instruction for LoadLocal {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let local = ctx
            .locals
            .get(self.index)
            .context(format!("no local @ {}", self.index))?;

        ctx.operands.push(local.clone());
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct StoreLocal {
    pub(crate) index: usize,
    pub(crate) store_next: bool,
}

impl Instruction for StoreLocal {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let locals = &mut ctx.locals;
        let target_index = if self.store_next {
            self.index + 1
        } else {
            self.index
        };
        let value = ctx.operands.pop().context("no operand to pop")?.clone();

        // Fill enough slots to be able to store at an arbitrary index
        // FIXME: We should probably keep a track of which locals are filled with "real"
        // values and which are just sentinels so we can provide more accurate diagnostics
        // for invalid store / get ops
        while locals.len() <= target_index {
            locals.push(RuntimeValue::null_ref());
        }

        locals[self.index] = value.clone();
        if self.store_next {
            locals[self.index + 1] = value;
        }
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct GetField {
    pub(crate) index: u16,
}

impl Instruction for GetField {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            name_and_type.descriptor.resolve().string(),
        );

        // The objectref, which must be of type reference but not an array
        // type, is popped from the operand stack.
        let objectref = arg!(ctx, "objectref" => Object);

        // dbg!(&objectref.borrow(), objectref.borrow().header().class().borrow(), &name, &descriptor, &objectref.borrow().class().borrow().instance_layout());

        // The value of the reference field in objectref is fetched and pushed onto the operand stack.
        let value = match FieldType::parse(descriptor.clone())? {
            FieldType::Base(ty) => match ty {
                BaseType::Boolean => {
                    let field: FieldRef<Bool> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();
                    RuntimeValue::Integral(field.copy_out().into())
                }
                BaseType::Char => {
                    let field: FieldRef<Char> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();

                    // TODO: Not entirely sure what the behaviour is supposed to be here wrt extension
                    let val = field.copy_out();
                    RuntimeValue::Integral((val as Int).into())
                }
                BaseType::Float => {
                    let field: FieldRef<Float> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();
                    RuntimeValue::Floating(field.copy_out().into())
                }
                BaseType::Double => {
                    let field: FieldRef<Double> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();
                    RuntimeValue::Floating(field.copy_out().into())
                }
                BaseType::Byte => {
                    let field: FieldRef<Byte> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();

                    // TODO: Not entirely sure what the behaviour is supposed to be here wrt extension
                    let val = field.copy_out();
                    RuntimeValue::Integral((val as Int).into())
                }
                BaseType::Short => {
                    let field: FieldRef<Short> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();

                    // TODO: Not entirely sure what the behaviour is supposed to be here wrt extension
                    let val = field.copy_out();
                    RuntimeValue::Integral((val as Int).into())
                }
                BaseType::Int => {
                    let field: FieldRef<Int> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();
                    RuntimeValue::Integral(field.copy_out().into())
                }
                BaseType::Long => {
                    let field: FieldRef<Long> =
                        objectref.borrow_mut().field((name, descriptor)).unwrap();
                    RuntimeValue::Integral(field.copy_out().into())
                }
                BaseType::Void => panic!("cannot read void field"),
            },
            FieldType::Object(_) => {
                let field: FieldRef<RefTo<Object>> =
                    objectref.borrow_mut().field((name, descriptor)).unwrap();
                RuntimeValue::Object(field.borrow().clone())
            }
            FieldType::Array(_) => {
                let field: FieldRef<RefTo<Object>> =
                    objectref.borrow_mut().field((name, descriptor)).unwrap();
                let value = field.borrow();
                RuntimeValue::Object(value.clone())
            }
        };

        ctx.operands.push(value);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct PutField {
    pub(crate) index: u16,
}

impl Instruction for PutField {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            FieldType::parse(name_and_type.descriptor.resolve().string())?,
        );

        // TODO: Type check & convert as needed

        //The value and objectref are popped from the operand stack.
        let value = pop!(ctx);
        let objectref = arg!(ctx, "objectref" => Object);

        macro_rules! set {
            (int $ty: ty, $o: expr, $name: expr, $desc: expr => $value: expr) => {{
                let field = $o.field::<$ty>(($name, $desc.to_string())).unwrap();
                field.write($value.as_integral().unwrap().value as $ty)
            }};
            (float $ty: ty, $o: expr, $name: expr, $desc: expr => $value: expr) => {{
                let field = $o.field::<$ty>(($name, $desc.to_string())).unwrap();
                field.write($value.as_floating().unwrap().value as $ty)
            }};
        }

        let o = objectref.borrow_mut();
        let name = name.clone();

        match descriptor {
            FieldType::Base(ref base) => match base {
                BaseType::Boolean => {
                    set!(int Bool, o, name, descriptor => value);
                }
                BaseType::Char => {
                    set!(int Char, o, name, descriptor => value);
                }
                BaseType::Float => {
                    set!(float Float, o, name, descriptor => value);
                }
                BaseType::Double => {
                    set!(float Double, o, name, descriptor => value);
                }
                BaseType::Byte => {
                    set!(int Byte, o, name, descriptor => value);
                }
                BaseType::Short => {
                    set!(int Short, o, name, descriptor => value);
                }
                BaseType::Int => {
                    set!(int Int, o, name, descriptor => value);
                }
                BaseType::Long => {
                    set!(int Long, o, name, descriptor => value);
                }
                BaseType::Void => todo!(),
            },
            FieldType::Object(_) => {
                let value_as_obj = value.as_object().unwrap();
                let value_as_obj = value_as_obj.clone();

                let field = o
                    .field::<RefTo<Object>>((name, descriptor.to_string()))
                    .unwrap();

                field.write(value_as_obj);
            }
            FieldType::Array(_) => {
                let obj = value.as_object().unwrap();
                let obj = obj.clone();

                // Safety: We have verified the type above.
                //         The component type does not matter because we are just
                //         Cloning the RefTo, not caring about the underlying array
                // let obj = unsafe { obj.cast::<Array<()>>() };
                // dbg!(&obj.borrow());

                let field = o
                    .field::<RefTo<Object>>((name, descriptor.to_string()))
                    .unwrap();

                field.write(obj);
            }
        };

        // Otherwise, the referenced field in objectref is set to value
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct GetStatic {
    pub(crate) index: u16,
}

impl Instruction for GetStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        // On successful resolution of the field, the class or interface that
        // declared the resolved field is initialized if that class or interface
        // has not already been initialized (§5.5).
        let class_name = field.class.resolve().name.resolve().string();
        let class = vm.class_loader.for_name(class_name.clone())?;

        vm.initialise_class(class.clone())?;

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            name_and_type.descriptor.resolve().string(),
        );

        let _class_name = class.borrow().name();

        // The value of the class or interface field is fetched and pushed onto the operand stack.

        // HACK: Hacking in inherited statics
        let mut cls = class.clone();
        loop {
            let field = cls
                .borrow()
                .static_field_info((name.clone(), descriptor.clone()));

            if let Some(f) = field {
                let value = f.value.clone().unwrap();
                ctx.operands.push(value);
                break
            } else {
                let sup = cls.borrow().super_class();
                vm.initialise_class(sup.clone())?;
                if sup.is_null() {
                    panic!("no field");
                }

                cls = sup;
            }
        };

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct PutStatic {
    pub(crate) index: u16,
}

impl Instruction for PutStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        // On successful resolution of the field, the class or interface that
        // declared the resolved field is initialized if that class or interface
        // has not already been initialized (§5.5).
        let class_name = field.class.resolve().name.resolve().string();
        let class = vm.class_loader.for_name(class_name.clone())?;
        vm.initialise_class(class.clone())?;

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            name_and_type.descriptor.resolve().string(),
        );

        // TODO: Type check & convert as needed
        let value = pop!(ctx);
        class
            .borrow_mut()
            .static_field_info_mut((name, descriptor))
            .unwrap()
            .value = Some(value);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Dup;

impl Instruction for Dup {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = pop!(ctx);
        ctx.operands.push(value.clone());
        ctx.operands.push(value);
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct DupX1;

impl Instruction for DupX1 {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value1 = pop!(ctx);
        let value2 = pop!(ctx);

        ctx.operands.push(value1.clone());
        ctx.operands.push(value2);
        ctx.operands.push(value1.clone());
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct DupX2;

impl Instruction for DupX2 {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = pop!(ctx);

        match value.computational_type() {
            ComputationalType::Category1 => {
                // Form 1:
                // ..., value3, value2, value1 →
                // ..., value1, value3, value2, value1
                // where value1, value2, and value3 are all values of a category 1
                // computational type (§2.11.1).
                let value1 = value;
                let value2 = pop!(ctx);
                let value3 = pop!(ctx);

                ctx.operands.push(value1.clone());
                ctx.operands.push(value3.clone());
                ctx.operands.push(value2.clone());
                ctx.operands.push(value1);
            }
            ComputationalType::Category2 => {
                // Form 2:
                // ..., value2, value1 →
                // ..., value1, value2, value1
                // where value1 is a value of a category 1 computational type and
                // value2 is a value of a category 2 computational type (§2.11.1).
                let value1 = value;
                let value2 = pop!(ctx);

                ctx.operands.push(value1.clone());
                ctx.operands.push(value2);
                ctx.operands.push(value1);
            }
        }

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Dup2;

impl Instruction for Dup2 {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = pop!(ctx);

        match value.computational_type() {
            ComputationalType::Category1 => {
                // Form 1:
                // ..., value2, value1 →
                // ..., value2, value1, value2, value1
                // where both value1 and value2 are values of a category 1
                // computational type (§2.11.1).
                let value1 = value;
                let value2 = pop!(ctx);

                ctx.operands.push(value2.clone());
                ctx.operands.push(value1.clone());
                ctx.operands.push(value2);
                ctx.operands.push(value1);
            }
            ComputationalType::Category2 => {
                // Form 2:
                // ..., value →
                // ..., value, value
                // where value is a value of a category 2 computational type
                // (§2.11.1).
                ctx.operands.push(value.clone());
                ctx.operands.push(value);
            }
        }

        Ok(Progression::Next)
    }
}
