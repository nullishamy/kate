


use crate::structs::types::{RefOrPrim, ReferenceType};
use crate::{CallSite, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;

use std::sync::Arc;
use tracing::{debug, warn};

pub fn areturn(_vm: &VM, ctx: &mut CallSite, _bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let mut sf = lock.pop().expect("call stack was empty?");

    let ref_value = sf.operand_stack.pop();

    if ref_value.is_none() {
        return Err(anyhow!("operand stack was empty"));
    }

    if !sf.operand_stack.is_empty() {
        warn!("attempted to areturn with values on the operand stack");
        sf.operand_stack.discard();
    }

    let ref_value = ref_value.unwrap();
    let ref_value = ref_value.as_reference();

    if ref_value.is_none() {
        return Err(anyhow!("ref value was not a reference"));
    }

    let ref_value = ref_value.unwrap();

    debug!("dropped a stackframe from the callstack");
    *ctx.pc.write() = 0;

    let caller = lock.peek_mut();

    if caller.is_none() {
        return Err(anyhow!("attempted to return from method with no caller?"));
    }

    let value = if ref_value.is_null() {
        ReferenceType::Null
    } else {
        ReferenceType::Class(Arc::clone(ref_value.as_class().expect("unreachable")))
    };

    let caller = caller.unwrap();

    debug!("pushed the reference onto the caller's op stack");
    caller.operand_stack.push(RefOrPrim::Reference(value));

    Ok(())
}
