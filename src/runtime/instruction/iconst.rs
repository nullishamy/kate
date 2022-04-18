use anyhow::Result;

use crate::runtime::stack::StackValue;
use crate::structs::types::PrimitiveWithValue;
use crate::{Context, VM};

pub fn iconst(_vm: &mut VM, ctx: &mut Context, value: i32) -> Result<()> {
    let mut st = ctx.thread.operand_stack.lock();
    st.push(StackValue::Primitive(PrimitiveWithValue::Int(value)));

    Ok(())
}
