use crate::{object::RuntimeValue, VM, Context};
use anyhow::{Result, Context as AnyhowContext, anyhow};
use parse::{classfile::Resolvable, pool::ConstantEntry};

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
    pub(crate) store_next: bool
}

impl Instruction for StoreLocal {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let locals = &mut ctx.locals;
        let target_index = if self.store_next { self.index + 1 } else { self.index };
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