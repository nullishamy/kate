use super::{Instruction, Progression};
use crate::{
    object::RuntimeValue,
    Context, VM,
};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use parse::{
    classfile::Resolvable, pool::ConstantEntry,
};

pub use super::binary::*;
pub use super::invoke::*;
pub use super::unary::*;

#[macro_export]
macro_rules! pop {
    ($ctx: expr) => {
        $ctx
            .operands
            .pop()
            .context("no value to pop from the operand stack")?
    };
}

#[macro_export]
macro_rules! arg {
    ($ctx: expr, $side: expr => i32) => {{
        let val = pop!($ctx);

        let val = val.as_integral().context(format!("{} was not an integral", $side))?;
        if val.ty != IntegralType::Int {
            return Err(anyhow!(format!("{} was not an int", $side)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => i64) => {{
        let val = pop!($ctx);

        let val = val.as_integral().context(format!("{} was not an integral", $side))?;
        if val.ty != IntegralType::Int {
            return Err(anyhow!(format!("{} was not an int", $side)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => f32) => {{
        let val = pop!($ctx);

        let val = val.as_floating().context(format!("{} was not a float", $side))?;
        if val.ty != FloatingType::Float {
            return Err(anyhow!(format!("{} was not a float", $side)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => f64) => {{
        let val = pop!($ctx);

        let val = val.as_floating().context(format!("{} was not a float", $side))?;
        if val.ty != FloatingType::Float {
            return Err(anyhow!(format!("{} was not a float", $side)));
        }

        val.clone()
    }};
}


#[derive(Debug)]
pub struct Nop {}
impl Instruction for Nop { }

#[derive(Debug)]
pub struct VoidReturn {}
impl Instruction for VoidReturn { }

#[derive(Debug)]
pub struct ValueReturn {
}

impl Instruction for ValueReturn {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let return_value = ctx.operands.pop().context("no return value popped")?;

        Ok(Progression::Return(Some(return_value)))
    }
}

#[derive(Debug)]
pub struct Goto {
    pub(crate) jump_to: i16
}

impl Instruction for Goto {
    fn handle(&self, _vm: &mut VM, _ctx: &mut Context) -> Result<Progression> {
        Ok(Progression::JumpRel(self.jump_to as i32))
    }
}

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

