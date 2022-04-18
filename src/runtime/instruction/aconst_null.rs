use crate::runtime::stack::StackValue;

use crate::structs::types::ReferenceType;
use crate::{Context, VM};
use anyhow::Result;
use bytes::Bytes;
use std::borrow::BorrowMut;
use tracing::debug;

pub fn aconst_null(_vm: &mut VM, ctx: &mut Context, _bytes: &mut Bytes) -> Result<()> {
    debug!(
        "pushing null to the operand stack ({:?})",
        ctx.thread.operand_stack.lock()
    );
    ctx.thread
        .operand_stack
        .lock()
        .borrow_mut()
        .push(StackValue::Reference(ReferenceType::Null));
    debug!(
        "pushed null to the operand stack ({:?})",
        ctx.thread.operand_stack.lock()
    );
    Ok(())
}
