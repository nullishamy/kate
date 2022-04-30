use crate::runtime::threading::thread::StackFrame;
use crate::{CallSite, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;

pub fn dup(vm: &VM, ctx: &mut CallSite, _bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let st = &mut sf.operand_stack;

    let v = st.peek();

    if v.is_none() {
        return Err(anyhow!("operand stack was empty"));
    }

    let v2 = v.unwrap().clone();

    st.push(v2);

    Ok(())
}
