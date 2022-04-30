use anyhow::{anyhow, Result};
use tracing::debug;

use crate::runtime::stack::StackValue;

use crate::{CallSite, VM};

pub fn astore(_vm: &VM, ctx: &mut CallSite, idx: u16) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    debug!("storing ref into {}", idx);

    let _ref = sf.operand_stack.pop();

    if let Some(local) = _ref {
        if let StackValue::Reference(_ref) = local {
            sf.locals.insert(idx as usize, StackValue::Reference(_ref));
            debug!("stored local");
            Ok(())
        } else {
            Err(anyhow!("value was not a reference, invalid bytecode"))
        }
    } else {
        Err(anyhow!("op stack was empty"))
    }
}
