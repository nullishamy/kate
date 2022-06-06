use anyhow::{anyhow, Result};
use tracing::debug;

use crate::runtime::stack::StackValue;

use crate::structs::types::PrimitiveWithValue;
use crate::{CallSite, Vm};

pub fn iload(_vm: &Vm, ctx: &mut CallSite, idx: u16) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    debug!("loading local @ idx {}", idx);

    let ops = &mut sf.operand_stack;
    let local = sf.locals.get(idx as usize);

    let local =
        local.ok_or_else(|| anyhow!("local @ idx {} did not exist, invalid bytecode", idx))?;

    if let StackValue::Primitive(ref_) = local {
        if let PrimitiveWithValue::Int(i) = ref_ {
            ops.push(StackValue::Primitive(PrimitiveWithValue::Int(*i)));
            debug!("pushed int {:?} to the op stack", ref_);
            Ok(())
        } else {
            Err(anyhow!(
                "local @ idx {} was not an int, invalid bytecode",
                idx
            ))
        }
    } else {
        Err(anyhow!(
            "local @ idx {} was not a primitive, invalid bytecode",
            idx
        ))
    }
}
