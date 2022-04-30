use crate::runtime::stack::StackValue;


use crate::structs::types::ReferenceType;
use crate::{CallSite, VM};
use anyhow::Result;
use bytes::Bytes;

use tracing::debug;

pub fn aconst_null(_vm: &VM, ctx: &mut CallSite, _bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    debug!("pushing null to the operand stack ({:?})", sf.operand_stack);
    sf.operand_stack
        .push(StackValue::Reference(ReferenceType::Null));
    debug!("pushed null to the operand stack ({:?})", sf.operand_stack);
    Ok(())
}
