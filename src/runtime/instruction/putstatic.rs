use crate::classfile::parse_helper::SafeBuf;

use crate::{CallSite, ClassLoader, Vm};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::borrow::BorrowMut;
use std::sync::Arc;

pub fn put_static(vm: &Vm, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let value = sf.operand_stack.pop();

    if value.is_none() {
        return Err(anyhow!("operand stack was empty"));
    }

    let idx = bytes.try_get_u16()?;

    let entry = ctx.class.const_pool.get(idx.into())?;
    let data = &entry.data.as_field_ref();

    if data.is_none() {
        return Err(anyhow!("entry was not a field, got {:?} instead", entry));
    }

    let nt = Arc::clone(&data.unwrap().name_and_type);
    let cls = Arc::clone(&data.unwrap().class);

    let cls = vm
        .system_classloader
        .write()
        .borrow_mut()
        .load_class(&cls.name.str)?;

    cls.run_clinit(
        vm,
        CallSite::new(
            Arc::clone(&cls),
            Arc::clone(&ctx.thread),
            Arc::clone(&ctx.method),
            None,
        ),
    )?;

    cls.fields
        .write()
        .statics
        .insert(nt.name.str.clone(), value.unwrap());

    Ok(())
}
