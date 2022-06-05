use crate::structs::types::{RefOrPrim, ReferenceType};
use crate::{CallSite, Vm};
use anyhow::{anyhow, Result};
use bytes::Bytes;

use std::sync::Arc;
use tracing::{debug, warn};

pub fn areturn(_vm: &Vm, ctx: &mut CallSite, _bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let mut sf = lock.pop().expect("call stack was empty?");

    let ref_value = sf.operand_stack.pop();
    let ref_value = ref_value.ok_or_else(|| anyhow!("operand stack was empty"))?;

    // TODO: I don't quite understand this, "operand stack was empty" is an error, and operand stack not empty is a warn?
    if !sf.operand_stack.is_empty() {
        warn!("attempted to areturn with values on the operand stack");
        sf.operand_stack.discard();
    }

    let ref_value = ref_value.as_reference();
    let ref_value = ref_value.ok_or_else(|| anyhow!("ref value was not a reference"))?;

    debug!("dropped a stackframe from the callstack");
    *ctx.pc.write() = 0;

    let caller = lock.peek_mut();
    let caller =
        caller.ok_or_else(|| anyhow!("attempted to return from method with no caller?"))?;

    let value = if ref_value.is_null() {
        ReferenceType::Null
    } else {
        ReferenceType::Class(Arc::clone(ref_value.as_class().expect("unreachable")))
    };

    debug!("pushed the reference onto the caller's op stack");
    caller.operand_stack.push(RefOrPrim::Reference(value));

    Ok(())
}
