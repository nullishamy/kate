use std::borrow::{Borrow, BorrowMut};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::classfile::parse_helper::SafeBuf;
use crate::{ClassLoader, Context, VM};

pub fn invoke_static(vm: &mut VM, ctx: &mut Context, bytes: &mut Bytes) -> Result<()> {
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

    cls.run_clinit(
        vm,
        Context {
            class: Arc::clone(&cls),
            thread: Arc::clone(&ctx.thread),
        },
    )?;

    let method = cls.methods.read().find(|m| m.name.str == nt.name.str);

    if method.is_none() {
        return Err(anyhow!("could not resolve static method '{}'", nt.name.str));
    }

    let method = method.unwrap();

    // interpret on the same thread, using a different class
    vm.interpret(
        method.borrow(),
        Context {
            class: Arc::clone(&cls),
            thread: Arc::clone(&ctx.thread),
        },
    )
}
