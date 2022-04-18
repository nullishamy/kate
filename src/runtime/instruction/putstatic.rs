use crate::classfile::parse_helper::SafeBuf;

use crate::{ClassLoader, Context, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::borrow::BorrowMut;
use std::sync::Arc;

pub fn put_static(vm: &mut VM, ctx: &mut Context, bytes: &mut Bytes) -> Result<()> {
    let value = ctx.thread.operand_stack.lock().pop();

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
        Context {
            class: Arc::clone(&cls),
            thread: Arc::clone(&ctx.thread),
        },
    )?;

    cls.fields
        .write()
        .statics
        .insert(nt.name.str.clone(), value.unwrap());

    Ok(())
}
