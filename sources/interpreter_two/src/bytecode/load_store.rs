use std::rc::Rc;

use crate::{
    arg,
    object::{statics::StaticFieldRef, RuntimeValue},
    pop, Context, VM,
};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use parse::{
    classfile::Resolvable,
    pool::{ConstantEntry, ConstantField},
};

use super::{Instruction, Progression};

#[derive(Debug)]
pub struct PushConst {
    pub(crate) value: RuntimeValue,
}

impl Instruction for PushConst {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        ctx.operands.push(self.value.clone());
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Ldc2W {
    pub(crate) index: u16,
}

impl Instruction for Ldc2W {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let value = ctx
            .class
            .read()
            .constant_pool()
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
            v => return Err(anyhow!("cannot load {:#?} with ldc2w", v)),
        };

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Ldc {
    pub(crate) index: u16,
}

impl Instruction for Ldc {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let value = ctx
            .class
            .read()
            .constant_pool()
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
                let obj = vm.interner.intern(str)?;

                ctx.operands.push(RuntimeValue::Object(obj))
            }
            ConstantEntry::Class(data) => {
                let class_name = data.name.resolve().string();
                let class = vm.class_loader.load_class(class_name)?;
                ctx.operands.push(RuntimeValue::Object(class));
            }
            v => return Err(anyhow!("cannot load {:#?} with ldc", v)),
        };

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct LoadLocal {
    pub(crate) index: usize,
}

impl Instruction for LoadLocal {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
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
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
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
            locals.push(RuntimeValue::Null);
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
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .read()
            .constant_pool()
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
        let value = objectref.read().get_instance_field((name, descriptor))?;
        ctx.operands.push(value);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct PutField {
    pub(crate) index: u16,
}

impl Instruction for PutField {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .read()
            .constant_pool()
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            name_and_type.descriptor.resolve().string(),
        );

        // TODO: Type check & convert as needed

        //The value and objectref are popped from the operand stack.
        let value = pop!(ctx);
        let objectref = arg!(ctx, "objectref" => Object);

        // Otherwise, the referenced field in objectref is set to value
        objectref
            .write()
            .set_instance_field((name, descriptor), value)?;

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct GetStatic {
    pub(crate) index: u16,
}

impl Instruction for GetStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .read()
            .constant_pool()
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        // On successful resolution of the field, the class or interface that
        // declared the resolved field is initialized if that class or interface
        // has not already been initialized (§5.5).
        let class_name = field.class.resolve().name.resolve().string();
        let class = vm.class_loader.load_class(class_name.clone())?;
        vm.initialise_class(Rc::clone(&class))?;

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            name_and_type.descriptor.resolve().string(),
        );

        // The value of the class or interface field is fetched and pushed onto the operand stack.
        let value = vm
            .statics
            .get_field(StaticFieldRef::new(
                class_name,
                name.clone(),
                descriptor.clone(),
            ))
            .context(format!(
                "no field {} ({}) in {}",
                name,
                descriptor,
                class.read().get_class_name().clone()
            ))?;

        ctx.operands.push(value);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct PutStatic {
    pub(crate) index: u16,
}

impl Instruction for PutStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The run-time constant pool entry at the index must be a symbolic
        // reference to a field (§5.1), which gives the name and descriptor of
        // the field as well as a symbolic reference to the class in which the
        // field is to be found.
        let field: ConstantField = ctx
            .class
            .read()
            .constant_pool()
            .address(self.index)
            .try_resolve()?;

        // The referenced field is resolved (§5.4.3.2).
        // TODO: Field resolution (through super classes etc)

        // On successful resolution of the field, the class or interface that
        // declared the resolved field is initialized if that class or interface
        // has not already been initialized (§5.5).
        let class_name = field.class.resolve().name.resolve().string();
        let class = vm.class_loader.load_class(class_name.clone())?;
        vm.initialise_class(Rc::clone(&class))?;

        let name_and_type = field.name_and_type.resolve();
        let (name, descriptor) = (
            name_and_type.name.resolve().string(),
            name_and_type.descriptor.resolve().string(),
        );

        // TODO: Type check & convert as needed
        let value = pop!(ctx);
        vm.statics.set_field(
            StaticFieldRef::new(class_name, name.clone(), descriptor.clone()),
            value,
        );

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct Dup;

impl Instruction for Dup {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let value = pop!(ctx);
        ctx.operands.push(value.clone());
        ctx.operands.push(value);
        Ok(Progression::Next)
    }
}
