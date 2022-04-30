use anyhow::Result;

use crate::runtime::stack::StackValue;
use crate::runtime::threading::thread::StackFrame;
use crate::structs::types::PrimitiveWithValue;
use crate::{CallSite, VM};

pub fn iconst(_vm: &VM, ctx: &mut CallSite, value: i32) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let st = &mut sf.operand_stack;
    st.push(StackValue::Primitive(PrimitiveWithValue::Int(value)));

    Ok(())
}
