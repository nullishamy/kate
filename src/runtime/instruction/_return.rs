
use crate::{CallSite, VM};
use anyhow::Result;
use bytes::Bytes;
use tracing::debug;

pub fn _return(_vm: &VM, ctx: &mut CallSite, _bytes: &mut Bytes) -> Result<()> {
    debug!("returning & discarding the op stack");
    let mut lock = ctx.thread.call_stack.lock();
    let mut sf = lock.pop().expect("call stack was empty?");
    sf.operand_stack.discard();
    *ctx.pc.write() = 0;
    debug!("dropped a stackframe from the callstack");
    Ok(())
}
