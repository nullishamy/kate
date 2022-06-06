use anyhow::{anyhow, Result};
use tracing::debug;

use crate::runtime::stack::StackValue;

use crate::{CallSite, Vm};

pub fn astore(_vm: &Vm, ctx: &mut CallSite, idx: u16) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    debug!("storing ref into {}", idx);

    let ref_ = sf.operand_stack.pop();
    let ref_ = ref_.ok_or_else(|| anyhow!("op stack was empty"))?;

    if let StackValue::Reference(ref_) = ref_ {
        sf.locals.insert(idx as usize, StackValue::Reference(ref_));
        debug!("stored local");
        Ok(())
    } else {
        Err(anyhow!("value was not a reference, invalid bytecode"))
    }
}
