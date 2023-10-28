use std::rc::Rc;

use super::{Instruction, Progression};
use crate::object::array::{Array, ArrayPrimitive, ArrayType};
use crate::object::numeric::IntegralType;
use crate::object::RuntimeValue;
use crate::{Context, VM};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use parking_lot::RwLock;
use parse::classfile::Resolvable;
use parse::pool::ConstantEntry;

pub use super::binary::*;
pub use super::invoke::*;
pub use super::load_store::*;
pub use super::unary::*;

#[macro_export]
macro_rules! pop {
    ($ctx: expr) => {
        $ctx.operands
            .pop()
            .context("no value to pop from the operand stack")?
    };
}

#[macro_export]
macro_rules! arg {
    ($ctx: expr, $side: expr => i32) => {{
        let val = pop!($ctx);

        let val = val
            .as_integral()
            .context(format!("{} was not an integral", $side))?;
        if val.ty != IntegralType::Int {
            return Err(anyhow!(format!("{} was not an int, got {:#?}", $side, val)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => i64) => {{
        let val = pop!($ctx);

        let val = val
            .as_integral()
            .context(format!("{} was not an integral", $side))?;
        if val.ty != IntegralType::Long {
            return Err(anyhow!(format!("{} was not a long, got {:#?}", $side, val)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => f32) => {{
        let val = pop!($ctx);

        let val = val
            .as_floating()
            .context(format!("{} was not a float", $side))?;
        if val.ty != FloatingType::Float {
            return Err(anyhow!(format!("{} was not a float", $side)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => f64) => {{
        let val = pop!($ctx);

        let val = val
            .as_floating()
            .context(format!("{} was not a float", $side))?;
        if val.ty != FloatingType::Float {
            return Err(anyhow!(format!("{} was not a float", $side)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => Object) => {{
        let val = pop!($ctx);

        let val = val
            .as_object()
            .context(format!("{} was not an object", $side))?;
        val.clone()
    }};
    ($ctx: expr, $side: expr => Array) => {{
        let val = pop!($ctx);

        let val = val
            .as_array()
            .context(format!("{} was not an array", $side))?;
        val.clone()
    }};
}

#[derive(Debug)]
pub struct Nop;
impl Instruction for Nop {}

#[derive(Debug)]
pub struct VoidReturn;
impl Instruction for VoidReturn {}

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
    pub(crate) jump_to: i16,
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

#[derive(Debug)]
pub struct ANewArray {
    pub(crate) type_index: u16,
}

impl Instruction for ANewArray {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The count must be of type int. It is popped off the operand stack.
        let count = arg!(ctx, "count" => i32);

        // The run-time constant pool entry at the index must
        // be a symbolic reference to a class, array, or interface type. The
        // named class, array, or interface type is resolved (§5.4.3.1).
        let ty: ConstantEntry = ctx
            .class
            .read()
            .constant_pool()
            .address(self.type_index)
            .resolve();

        let array_ty = match ty {
            ConstantEntry::Class(data) => {
                let class_name = data.name.resolve().string();
                let cls = vm.class_loader.load_class(class_name)?;
                ArrayType::Object(cls)
            }
            e => return Err(anyhow!("{:#?} cannot be used as an array type", e)),
        };

        // All components of the new array are initialized to null, the default value for reference types (§2.4).
        let mut values = Vec::with_capacity(count.value as usize);
        values.resize_with(count.value as usize, || RuntimeValue::Null);

        // A new array with components of that type, of length count, is allocated
        // from the garbage-collected heap.
        let array = Array {
            ty: array_ty,
            values,
        };

        //  and a arrayref to this new array object is pushed onto the operand stack.
        let array_ref = Rc::new(RwLock::new(array));
        ctx.operands.push(RuntimeValue::Array(array_ref));

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct NewArray {
    pub(crate) type_tag: u8,
}

impl Instruction for NewArray {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        // The count must be of type int. It is popped off the operand stack.
        let count = arg!(ctx, "count" => i32);

        // The atype is a code that indicates the type of array to create.
        let atype = ArrayPrimitive::from_tag(self.type_tag)?;
        let array_ty = ArrayType::Primitive(atype);

        let mut values = Vec::with_capacity(count.value as usize);
        // TODO: Proper default values
        values.resize_with(count.value as usize, || {
            RuntimeValue::Integral((0_i32).into())
        });

        // A new array whose components are of type atype and of length
        // count is allocated from the garbage-collected heap.
        let array = Array {
            ty: array_ty,
            values,
        };

        // and an arrayref to this new array object is pushed onto the operand stack.
        let array_ref = Rc::new(RwLock::new(array));
        ctx.operands.push(RuntimeValue::Array(array_ref));

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct ArrayStore;

impl Instruction for ArrayStore {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let value = pop!(ctx);
        let index = arg!(ctx, "index" => i32);
        let array = arg!(ctx, "array" => Array);

        let mut array = array.write();
        array.values[index.value as usize] = value;

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct ArrayLoad;

impl Instruction for ArrayLoad {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
        let index = arg!(ctx, "index" => i32);
        let array = arg!(ctx, "array" => Array);

        let array = array.read();
        let value = array.values[index.value as usize].clone();
        ctx.operands.push(value);

        Ok(Progression::Next)
    }
}
