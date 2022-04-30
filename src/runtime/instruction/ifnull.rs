

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::bytecode::args::Args;


use crate::{CallSite, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;


use tracing::debug;

pub fn ifnull(vm: &VM, ctx: &mut CallSite, args: &mut Args, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let branch_if_null = bytes.try_get_u16()? - 1;

    let obj_ref = sf.operand_stack.pop();

    if obj_ref.is_none() {
        return Err(anyhow!("operand stack was empty"));
    }

    let obj_ref = obj_ref.unwrap();
    let obj_ref = obj_ref.as_reference();

    if obj_ref.is_none() {
        return Err(anyhow!("obj ref was a primitive, expected reference"));
    }

    let obj_ref = obj_ref.unwrap();
    let obj_ref = obj_ref.as_class();

    if obj_ref.is_none() {
        // ifnull, modify by offset
        debug!("value was null");
        *ctx.pc.write() += branch_if_null as usize;
    } else {
        debug!("value was not null");
    }

    debug!("continuing from {}", *ctx.pc.read());

    // drop before we restart interpretation, deadlock is possible here
    drop(lock);

    // continue interpretation, if it was null the offset was applied
    // and we should continue from there

    let new_ctx = ctx.clone();

    // args are relatively cheap to clone, just need a copy of the vec
    // which shouldnt be too expensive to allocate
    let new_args = args.clone();

    vm.interpret(new_ctx, new_args, true)?;
    Ok(())
}
