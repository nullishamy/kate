use anyhow::{anyhow, Result};
use tracing::debug;

use crate::runtime::stack::StackValue;

use crate::{CallSite, Vm};

pub fn aload(_vm: &Vm, ctx: &mut CallSite, idx: u16) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    debug!("loading local @ idx {}", idx);

    let ops = &mut sf.operand_stack;
    let local = sf.locals.get(idx as usize);

    if let Some(local) = local {
        if let StackValue::Reference(ref_) = local {
            ops.push(StackValue::Reference(ref_.clone()));
            debug!("pushed local to the op stack");
            Ok(())
        } else {
            Err(anyhow!(
                "local @ idx {} was not a reference, invalid bytecode",
                idx
            ))
        }
    } else {
        Err(anyhow!(
            "local @ idx {} did not exist, invalid bytecode",
            idx
        ))
    }
}
