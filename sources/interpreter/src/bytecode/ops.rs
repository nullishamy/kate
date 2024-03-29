use super::{Instruction, Progression};
use crate::error::{Throwable, VMError};
use crate::object::builtins::{Array, ArrayPrimitive, ArrayType, Object};
use crate::object::layout::types::{Bool, Byte, Char, Double, Float, Int, Long, Short};
use crate::object::mem::RefTo;
use crate::object::numeric::IntegralType;
use crate::object::runtime::RuntimeValue;
use crate::{internal, Context, VM};
use anyhow::Context as AnyhowContext;

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
            return Err($crate::internal!(format!(
                "{} was not an int, got {:#?}",
                $side, val
            )));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => i64) => {{
        let val = pop!($ctx);

        let val = val
            .as_integral()
            .context(format!("{} was not an integral", $side))?;
        if val.ty != IntegralType::Long {
            return Err($crate::internal!(format!(
                "{} was not a long, got {:#?}",
                $side, val
            )));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => f32) => {{
        let val = pop!($ctx);

        let val = val
            .as_floating()
            .context(format!("{} was not a floating", $side))?;
        if val.ty != FloatingType::Float {
            return Err($crate::internal!(format!("{} was not a float", $side)));
        }

        val.clone()
    }};
    ($ctx: expr, $side: expr => f64) => {{
        let val = pop!($ctx);

        let val = val
            .as_floating()
            .context(format!("{} was not a floating", $side))?;
        if val.ty != FloatingType::Double {
            return Err($crate::internal!(format!("{} was not a double", $side)));
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
    ($ctx: expr, $side: expr => Array<$comp: ty>) => {{
        let val = pop!($ctx);

        let val = val
            .as_object()
            .context(format!("{} was not an object", $side))?;

        // Safety: We have checked the type.
        unsafe { val.cast::<Array<$comp>>().clone() }
    }};
}

#[derive(Debug)]
pub struct Nop;
impl Instruction for Nop {}

#[derive(Debug)]
pub struct VoidReturn;
impl Instruction for VoidReturn {
    fn handle(&self, _vm: &mut VM, _ctx: &mut Context) -> Result<Progression, Throwable> {
        Ok(Progression::Return(None))
    }
}

#[derive(Debug)]
pub struct ValueReturn;

impl Instruction for ValueReturn {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let return_value = ctx.operands.pop().context("no return value popped")?;

        Ok(Progression::Return(Some(return_value)))
    }
}

#[derive(Debug)]
pub struct Goto {
    pub(crate) jump_to: i16,
}

impl Instruction for Goto {
    fn handle(&self, _vm: &mut VM, _ctx: &mut Context) -> Result<Progression, Throwable> {
        Ok(Progression::JumpRel(self.jump_to as i32))
    }
}

#[derive(Debug)]
pub struct ArrayLength;

impl Instruction for ArrayLength {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The arrayref must be of type reference and must refer to an array. It is popped from the operand stack.
        let array = arg!(ctx, "array" => Array<()>);

        // The length of the array it references is determined.
        let len = array.borrow().len() as i32;

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
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The count must be of type int. It is popped off the operand stack.
        let count = arg!(ctx, "count" => i32);

        // The run-time constant pool entry at the index must
        // be a symbolic reference to a class, array, or interface type. The
        // named class, array, or interface type is resolved (§5.4.3.1).
        let ty: ConstantEntry = ctx
            .class
            .borrow()
            .class_file()
            .constant_pool
            .address(self.type_index)
            .resolve();

        let (array_ty, array_ty_name) = match ty {
            ConstantEntry::Class(data) => {
                let class_name = data.name.resolve().string();
                let cls = vm.class_loader.for_name(class_name)?;
                let name = cls.borrow().name().clone();

                (ArrayType::Object(cls), name)
            }
            e => return Err(internal!("{:#?} cannot be used as an array type", e)),
        };

        // All components of the new array are initialized to null, the default value for reference types (§2.4).
        let mut values: Vec<RefTo<Object>> = Vec::with_capacity(count.value as usize);
        values.resize_with(count.value as usize, || RefTo::null());

        // A new array with components of that type, of length count, is allocated
        // from the garbage-collected heap.
        let array = Array::<RefTo<Object>>::from_vec(array_ty, array_ty_name.to_string(), values);

        //  and a arrayref to this new array object is pushed onto the operand stack.
        ctx.operands.push(RuntimeValue::Object(array.erase()));

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct NewArray {
    pub(crate) type_tag: u8,
}

impl Instruction for NewArray {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        // The count must be of type int. It is popped off the operand stack.
        let count = arg!(ctx, "count" => i32);

        // The atype is a code that indicates the type of array to create.
        let atype = ArrayPrimitive::from_tag(self.type_tag)?;
        let array_ty = ArrayType::Primitive(atype);

        // A new array whose components are of type atype and of length
        // count is allocated from the garbage-collected heap.
        let array = match &array_ty {
            ArrayType::Primitive(ty) => match ty {
                ArrayPrimitive::Bool => {
                    let values: Vec<Bool> = vec![0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Bool>::from_vec(array_ty, "[Z".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Char => {
                    let values: Vec<Char> = vec![0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Char>::from_vec(array_ty, "[C".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Float => {
                    let values: Vec<Float> = vec![0.0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Float>::from_vec(array_ty, "[F".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Double => {
                    let values: Vec<Double> = vec![0.0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Double>::from_vec(array_ty, "[D".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Byte => {
                    let values: Vec<Byte> = vec![0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Byte>::from_vec(array_ty, "[B".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Short => {
                    let values: Vec<Short> = vec![0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Short>::from_vec(array_ty, "[S".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Int => {
                    let values: Vec<Int> = vec![0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Int>::from_vec(array_ty, "[I".to_string(), values).erase(),
                    )
                }
                ArrayPrimitive::Long => {
                    let values: Vec<Long> = vec![0; count.value as usize];
                    RuntimeValue::Object(
                        Array::<Long>::from_vec(array_ty, "[J".to_string(), values).erase(),
                    )
                }
            },
            _ => unreachable!(),
        };

        // and an arrayref to this new array object is pushed onto the operand stack.
        ctx.operands.push(array);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct ArrayStore {
    pub(crate) ty: ArrayType,
}

impl Instruction for ArrayStore {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let value = pop!(ctx);
        let index = arg!(ctx, "index" => i32);
        match &self.ty {
            ArrayType::Object(_) => {
                let array = arg!(ctx, "array" => Array<RefTo<Object>>);
                let array = array.borrow_mut().slice_mut();
                let value = value.as_object().expect("array store exception").clone();
                array[index.value as usize] = value;
            }
            ArrayType::Primitive(ty) => match ty {
                // ArrayPrimitive::Bool => todo!(),
                // ArrayPrimitive::Char => todo!(),
                // ArrayPrimitive::Float => todo!(),
                // ArrayPrimitive::Double => todo!(),
                // ArrayPrimitive::Byte => todo!(),
                // ArrayPrimitive::Short => todo!(),
                // ArrayPrimitive::Int => todo!(),
                ArrayPrimitive::Long => {
                    let array = arg!(ctx, "array" => Array<Long>);
                    let array = array.borrow_mut().slice_mut();
                    let value = value.as_integral().expect("array store exception").value;
                    array[index.value as usize] = value
                }
                ArrayPrimitive::Double => {
                    let array = arg!(ctx, "array" => Array<Double>);
                    let array = array.borrow_mut().slice_mut();
                    let value = value.as_floating().expect("array store exception").value;
                    array[index.value as usize] = value
                }
                ArrayPrimitive::Byte => {
                    let array = arg!(ctx, "array" => Array<Byte>);
                    let array = array.borrow_mut().slice_mut();
                    let value = value.as_integral().expect("array store exception").value;
                    array[index.value as usize] = value as Byte
                }
                ArrayPrimitive::Char => {
                    let array = arg!(ctx, "array" => Array<Char>);
                    let array = array.borrow_mut().slice_mut();
                    let value = value.as_integral().expect("array store exception").value;
                    array[index.value as usize] = value as Char
                }
                ArrayPrimitive::Int => {
                    let array = arg!(ctx, "array" => Array<Int>);
                    let array = array.borrow_mut().slice_mut();
                    let value = value.as_integral().expect("array store exception").value;
                    array[index.value as usize] = value as Int
                }
                ty => panic!("cannot encode {:#?}", ty),
            },
        };

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct ArrayLoad {
    pub(crate) ty: ArrayType,
}

impl Instruction for ArrayLoad {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        let index = arg!(ctx, "index" => i32);

        let value = match &self.ty {
            ArrayType::Object(_) => {
                let array = arg!(ctx, "array" => Array<RefTo<Object>>);
                let array = array.borrow().slice();
                let value = array[index.value as usize].clone();
                RuntimeValue::Object(value)
            }
            ArrayType::Primitive(ty) => match ty {
                // ArrayPrimitive::Bool => todo!(),
                ArrayPrimitive::Char => {
                    let array = arg!(ctx, "array" => Array<Char>);
                    let array = array.borrow().slice();

                    if index.value >= array.len() as i64 {
                        return Ok(Progression::Throw(
                            vm.make_error(VMError::ArrayIndexOutOfBounds { at: index.value })?,
                        ));
                    }

                    let value = array[index.value as usize];

                    // TODO: Sign extension here?
                    RuntimeValue::Integral((value as i32).into())
                }
                // ArrayPrimitive::Float => todo!(),
                ArrayPrimitive::Double => {
                    let array = arg!(ctx, "array" => Array<Double>);
                    let array = array.borrow().slice();
                    let value = array[index.value as usize];

                    RuntimeValue::Floating(value.into())
                }
                ArrayPrimitive::Byte => {
                    let array = arg!(ctx, "array" => Array<Byte>);
                    let array = array.borrow().slice();
                    let value = array[index.value as usize];

                    // TODO: Sign extension here
                    RuntimeValue::Integral((value as i32).into())
                }
                ArrayPrimitive::Long => {
                    let array = arg!(ctx, "array" => Array<Long>);
                    let array = array.borrow().slice();
                    let value = array[index.value as usize];

                    RuntimeValue::Integral(value.into())
                }
                // ArrayPrimitive::Short => todo!(),
                // ArrayPrimitive::Int => todo!(),
                // ArrayPrimitive::Long => todo!(),
                ty => panic!("cannot encode {:#?}", ty),
            },
        };

        ctx.operands.push(value);

        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct MonitorEnter;
impl Instruction for MonitorEnter {
    // TODO: Support when we support MT
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        pop!(ctx);
        Ok(Progression::Next)
    }
}

#[derive(Debug)]
pub struct MonitorExit;
impl Instruction for MonitorExit {
    // TODO: Support when we support MT
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression, Throwable> {
        pop!(ctx);
        Ok(Progression::Next)
    }
}
