use super::{Instruction, Progression};
use crate::arg;
use crate::pop;
use crate::Context;
use crate::Interpreter;
use anyhow::Context as AnyhowContext;
use parse::{
    classfile::Resolvable,
    pool::{ConstantEntry, ConstantField},
};
use runtime::error::Throwable;
use runtime::internal;
use runtime::object::builtins::Object;
use runtime::object::interner::intern_string;
use runtime::object::layout::types::Bool;
use runtime::object::layout::types::Byte;
use runtime::object::layout::types::Char;
use runtime::object::layout::types::Double;
use runtime::object::layout::types::Float;
use runtime::object::layout::types::Int;
use runtime::object::layout::types::Long;
use runtime::object::layout::types::Short;
use runtime::object::mem::FieldRef;
use runtime::object::mem::RefTo;
use runtime::object::value::ComputationalType;
use runtime::object::value::RuntimeValue;
use support::descriptor::{BaseType, FieldType};

#[derive(Debug)]
pub struct PushConst {
    pub(crate) value: RuntimeValue,
}

impl Instruction for PushConst {
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        ctx.operands.push(self.value.clone());
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Ldc2W {
    pub(crate) index: u16,
}

impl Instruction for Ldc2W {
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = ctx
            .class
            .unwrap_ref()
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
    fn handle(&self, vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = ctx
            .class
            .unwrap_ref()
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

                let class_name = FieldType::parse(class_name.clone())
                    .or_else(|_| FieldType::parse(format!("L{};", class_name)))?;

                let class = vm.class_loader().for_name(class_name)?;

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
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
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
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        let locals = &mut ctx.locals;
        let target_index = if self.store_next {
            self.index + 1
        } else {
            self.index
        };
        let value = ctx.operands.pop().context("no operand to pop")?.clone();

        // Fill enough slots to be able to store at an arbitrary index
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
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .unwrap_ref()
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

        // The value of the reference field in objectref is fetched and pushed onto the operand stack.
        let value = match FieldType::parse(descriptor.clone())? {
            FieldType::Base(ty) => match ty {
                BaseType::Boolean => {
                    let field: FieldRef<Bool> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();
                    RuntimeValue::Integral(field.copy_out().into())
                }
                BaseType::Char => {
                    let field: FieldRef<Char> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();

                    // TODO: Not entirely sure what the behaviour is supposed to be here wrt extension
                    let val = field.copy_out();
                    RuntimeValue::Integral((val as Int).into())
                }
                BaseType::Float => {
                    let field: FieldRef<Float> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();
                    RuntimeValue::Floating(field.copy_out().into())
                }
                BaseType::Double => {
                    let field: FieldRef<Double> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();
                    RuntimeValue::Floating(field.copy_out().into())
                }
                BaseType::Byte => {
                    let field: FieldRef<Byte> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();

                    // TODO: Not entirely sure what the behaviour is supposed to be here wrt extension
                    let val = field.copy_out();
                    RuntimeValue::Integral((val as Int).into())
                }
                BaseType::Short => {
                    let field: FieldRef<Short> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();

                    // TODO: Not entirely sure what the behaviour is supposed to be here wrt extension
                    let val = field.copy_out();
                    RuntimeValue::Integral((val as Int).into())
                }
                BaseType::Int => {
                    let field: FieldRef<Int> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();

                    RuntimeValue::Integral(field.copy_out().into())
                }
                BaseType::Long => {
                    let field: FieldRef<Long> =
                        objectref.unwrap_ref().field((name, descriptor)).unwrap();
                    RuntimeValue::Integral(field.copy_out().into())
                }
                BaseType::Void => return Err(internal!("cannot read void field")),
            },
            FieldType::Object(_) => {
                let field: FieldRef<RefTo<Object>> =
                    objectref.unwrap_ref().field((name, descriptor)).unwrap();
                RuntimeValue::Object(field.unwrap_ref().clone())
            }
            FieldType::Array(_) => {
                let field: FieldRef<RefTo<Object>> =
                    objectref.unwrap_ref().field((name, descriptor)).unwrap();
                let value = field.unwrap_ref();
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
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .unwrap_ref()
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

        let o = objectref.unwrap_ref();
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
    fn handle(&self, vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .unwrap_ref()
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
        let class = vm
            .class_loader()
            .for_name(format!("L{};", class_name).into())?;

        vm.initialise_class(class.clone())?;

        let name_and_type = field.name_and_type.resolve();
        let name = name_and_type.name.resolve().string();

        let _class_name = class.unwrap_ref().name();

        // The value of the class or interface field is fetched and pushed onto the operand stack.

        // HACK: Hacking in inherited statics
        // This should be handled by the field resolution algorithm, as suggested above.
        // This algorithm is similar to the method resolution algorithms already implemented for the invoke* instructions
        let mut cls = class.clone();
        loop {
            let statics = cls.unwrap_ref().statics();
            let statics = statics.read();
            let field = statics.get(&name);

            if let Some(f) = field {
                let value = f.value.clone().unwrap();
                ctx.operands.push(value);
                break;
            } else {
                let sup = cls.unwrap_ref().super_class();
                // We searched every class and could not find the static, this is a fault with the classfile
                // or our parser / layout mechanisms
                if sup.is_null() {
                    return Err(internal!(
                        "could not locate static field in class or super class(es)"
                    ));
                }

                vm.initialise_class(sup.clone())?;

                drop(statics);

                cls = sup;
            }
        }

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct PutStatic {
    pub(crate) index: u16,
}

impl Instruction for PutStatic {
    fn handle(&self, vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .unwrap_ref()
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
        let class = vm
            .class_loader()
            .for_name(format!("L{};", class_name).into())?;
        vm.initialise_class(class.clone())?;

        let name_and_type = field.name_and_type.resolve();
        let name = name_and_type.name.resolve().string();

        // TODO: Type check & convert as needed
        let value = pop!(ctx);
        let statics = class.unwrap_ref().statics();
        let mut statics = statics.write();

        statics.get_mut(&name).unwrap().value = Some(value);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Dup;

impl Instruction for Dup {
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = pop!(ctx);
        ctx.operands.push(value.clone());
        ctx.operands.push(value);
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct DupX1;

impl Instruction for DupX1 {
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
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
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
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
    fn handle(&self, _vm: &mut Interpreter, ctx: &mut Context) -> Result<Progression, Throwable> {
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
