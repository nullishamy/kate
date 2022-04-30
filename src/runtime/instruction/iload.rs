use anyhow::{anyhow, Result};
use tracing::debug;

use crate::runtime::stack::StackValue;
use crate::runtime::threading::thread::StackFrame;
use crate::structs::types::PrimitiveWithValue;
use crate::{CallSite, VM};

pub fn iload(vm: &VM, ctx: &mut CallSite, idx: u16) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    debug!("loading local @ idx {}", idx);

    let ops = &mut sf.operand_stack;
    let local = sf.locals.get(idx as usize);

    if let Some(local) = local {
        if let StackValue::Primitive(_ref) = local {
            if let PrimitiveWithValue::Int(i) = _ref {
                ops.push(StackValue::Primitive(PrimitiveWithValue::Int(*i)));
                debug!("pushed int {:?} to the op stack", _ref);
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
    } else {
        Err(anyhow!(
            "local @ idx {} did not exist, invalid bytecode",
            idx
        ))
    }
}
