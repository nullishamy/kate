use super::{Instruction, Progression};
use crate::object::RuntimeValue;
use crate::{
    Context, VM,
};
use anyhow::{Context as AnyhowContext, Result};

pub use super::binary::*;
pub use super::invoke::*;
pub use super::unary::*;
pub use super::load_store::*;

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
    ($ctx: expr, $side: expr => Object) => {{
        let val = pop!($ctx);

        let val = val.as_object().context(format!("{} was not an object", $side))?;
        val.clone()
    }};
    ($ctx: expr, $side: expr => Array) => {{
        let val = pop!($ctx);

        let val = val.as_array().context(format!("{} was not an array", $side))?;
        val.clone()
    }};
}


#[derive(Debug)]
pub struct Nop;
impl Instruction for Nop { }

#[derive(Debug)]
pub struct VoidReturn;
impl Instruction for VoidReturn { }

#[derive(Debug)]
pub struct ValueReturn;

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
pub struct ArrayLength;

impl Instruction for ArrayLength {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The arrayref must be of type reference and must refer to an array. It is popped from the operand stack.
        let array = arg!(ctx, "array" => Array);

        // The length of the array it references is determined.
        let len = array.read().len() as i32;

        // That length is pushed onto the operand stack as an int.
        ctx.operands.push(RuntimeValue::Integral(len.into()));
        Ok(Progression::Next)
    }
}