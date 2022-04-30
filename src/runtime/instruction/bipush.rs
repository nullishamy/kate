use crate::runtime::stack::StackValue;

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::threading::thread::StackFrame;
use crate::structs::types::{PrimitiveWithValue, ReferenceType};
use crate::{CallSite, VM};
use anyhow::Result;
use bytes::Bytes;
use std::borrow::BorrowMut;
use tracing::debug;

pub fn bipush(vm: &VM, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let byte = bytes.try_get_i8()? as i32;

    sf.operand_stack
        .push(StackValue::Primitive(PrimitiveWithValue::Int(byte)));

    Ok(())
}
