use std::borrow::BorrowMut;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use tracing::debug;

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::instruction::util::create_args;

use crate::{CallSite, ClassLoader, Vm};

pub fn invoke_static(vm: &Vm, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let idx = bytes.try_get_u16()?;

    let entry = ctx.class.const_pool.get(idx as usize)?;

    let m_ref = &entry.data.as_method_ref();
    let i_ref = &entry.data.as_interface_method_ref();

    if m_ref.is_none() && i_ref.is_none() {
        return Err(anyhow!(
            "expected method ref or interface method ref, got {:?}",
            entry
        ));
    }

    // unwrapping as the None case is handled above
    let nt = m_ref
        .map(|m| m.name_and_type.clone())
        .or_else(|| i_ref.map(|i| i.name_and_type.clone()))
        .unwrap();

    let cls = m_ref
        .map(|m| m.class.clone())
        .or_else(|| i_ref.map(|i| i.class.clone()))
        .unwrap();

    let cls = vm
        .system_classloader
        .write()
        .borrow_mut()
        .load_class(&cls.name.str)?;

    let method = cls.methods.read().find(|m| m.name.str == nt.name.str);

    let method =
        method.ok_or_else(|| anyhow!("could not resolve static method '{}'", nt.name.str))?;

    debug!(
        "INVOKESTATIC: {} {} {}",
        idx, cls.this_class.name.str, method.name.str
    );

    let args = create_args(&method.descriptor, &mut sf.operand_stack)?;

    if cls.requires_clinit() {
        drop(lock);
        cls.run_clinit(
            vm,
            CallSite::new(
                Arc::clone(&cls),
                Arc::clone(&ctx.thread),
                Arc::clone(&ctx.method),
                None,
            ),
        )?;
    } else {
        drop(lock)
    }

    // interpret on the same thread, using a different class
    vm.interpret(
        CallSite::new(Arc::clone(&cls), Arc::clone(&ctx.thread), method, None),
        args,
        false,
    )
}
