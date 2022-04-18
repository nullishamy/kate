use crate::{Context, VM};
use anyhow::Result;
use bytes::Bytes;
use std::borrow::BorrowMut;
use tracing::debug;

pub fn _return(_vm: &mut VM, ctx: &mut Context, _bytes: &mut Bytes) -> Result<()> {
    debug!("returning & discarding the op stack");
    ctx.thread.operand_stack.lock().borrow_mut().discard();
    Ok(())
}
