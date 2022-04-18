use anyhow::{anyhow, Result};
use tracing::debug;

use crate::runtime::stack::StackValue;
use crate::{Context, VM};

pub fn aload(_vm: &mut VM, ctx: &mut Context, idx: u16) -> Result<()> {
    debug!("loading local @ idx {}", idx);

    let _lock = ctx.thread.locals.lock();
    let local = _lock.get(idx as usize);

    if let Some(local) = local {
        if let StackValue::Reference(_ref) = local {
            let mut ops = ctx.thread.operand_stack.lock();
            ops.push(StackValue::Reference(_ref.clone()));
            debug!("pushed local {:?} to the op stack ({:?})", _ref, ops);
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
