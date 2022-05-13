use crate::runtime::stack::StackValue;

use crate::classfile::parse_helper::SafeBuf;

use crate::structs::types::PrimitiveWithValue;
use crate::{CallSite, Vm};
use anyhow::Result;
use bytes::Bytes;

pub fn bipush(_vm: &Vm, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let byte = bytes.try_get_i8()? as i32;

    sf.operand_stack
        .push(StackValue::Primitive(PrimitiveWithValue::Int(byte)));

    Ok(())
}
