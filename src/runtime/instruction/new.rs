use crate::runtime::stack::StackValue;

use crate::classfile::parse_helper::SafeBuf;
use crate::structs::types::ReferenceType;
use crate::{ClassLoader, Context, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::borrow::BorrowMut;
use std::sync::Arc;

pub fn new(vm: &mut VM, ctx: &mut Context, bytes: &mut Bytes) -> Result<()> {
    let idx = bytes.try_get_u16()?;

    let entry = ctx.class.const_pool.get(idx.into())?;

    let data = &entry.data.as_class();

    if data.is_none() {
        return Err(anyhow!("expected class data, got {:?}", entry));
    }

    let cls = data.unwrap();

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

    let obj = cls.new_instance(vm)?;

    ctx.thread
        .operand_stack
        .lock()
        .push(StackValue::Reference(ReferenceType::Class(obj)));
    Ok(())
}
