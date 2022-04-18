use crate::{Context, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;

pub fn dup(_vm: &mut VM, ctx: &mut Context, _bytes: &mut Bytes) -> Result<()> {
    let mut st = ctx.thread.operand_stack.lock();

    let v = st.peek();

    if v.is_none() {
        return Err(anyhow!("operand stack was empty"));
    }

    let v2 = v.unwrap().clone();

    st.push(v2);

    Ok(())
}
