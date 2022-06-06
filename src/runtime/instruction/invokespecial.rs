use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::instruction::util::create_args;

use crate::{CallSite, ClassLoader, Vm};
use anyhow::{anyhow, Result};
use bytes::Bytes;

use std::sync::Arc;
use tracing::debug;

pub fn invoke_special(vm: &Vm, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let idx = bytes.try_get_u16()?;

    let entry = ctx.class.const_pool.get(idx.into())?;
    let data = entry.data.as_method_ref();

    let method = data.ok_or_else(|| anyhow!("expected method ref, got {:?}", entry))?;

    let nt = Arc::clone(&method.name_and_type);
    let cls = Arc::clone(&method.class);
    let cls = vm.system_classloader.write().load_class(&cls.name.str)?;

    let method = cls.methods.read().find(|f| f.name.str == nt.name.str);

    let method = method.ok_or_else(|| {
        anyhow!(
            "could not find method {} for class {}",
            nt.name.str,
            cls.this_class.name.str
        )
    })?;

    debug!(
        "INVOKESPECIAL: {} {} {}",
        idx, cls.this_class.name.str, method.name.str
    );

    let args = create_args(&method.descriptor, &mut sf.operand_stack)?;

    // pop the value since its passed as the first arg to the function, to represent "this"
    // it should be the only value left after args have been popped off
    let obj_ref = sf.operand_stack.pop();

    let obj_ref = obj_ref.ok_or_else(|| anyhow!("operand stack was empty"))?;
    let obj_ref = obj_ref.as_reference();

    let obj_ref = obj_ref.ok_or_else(|| anyhow!("obj ref was a primitive, expected reference"))?;
    let obj_ref = obj_ref
        .as_class()
        .ok_or_else(|| anyhow!("obj ref was a null, expected class"))?;

    let obj_ref = Arc::clone(obj_ref);

    // drop the lock before re-interpreting in case 'method' invokes invokespecial again
    // if we didnt do this, we could deadlock
    drop(lock);
    if cls.requires_clinit() {
        cls.run_clinit(
            vm,
            CallSite::new(
                Arc::clone(&cls),
                Arc::clone(&ctx.thread),
                Arc::clone(&ctx.method),
                Some(Arc::clone(&obj_ref)),
            ),
        )?;
    }

    //TODO: respect polymorphic calls
    // replace '&cls' with obj_ref's class once we figure out polymorphism
    vm.interpret(
        CallSite::new(
            Arc::clone(&cls),
            Arc::clone(&ctx.thread),
            method,
            Some(obj_ref),
        ),
        args,
        false,
    )?;

    Ok(())
}
