use crate::{CallSite, Vm};
use anyhow::{anyhow, Result};
use bytes::Bytes;

pub fn dup(_vm: &Vm, ctx: &mut CallSite, _bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let st = &mut sf.operand_stack;

    let v = st.peek();
    let v = v.ok_or_else(|| anyhow!("operand stack was empty"))?.clone();

    st.push(v);

    Ok(())
}
