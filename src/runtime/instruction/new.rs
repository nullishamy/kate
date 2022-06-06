use crate::runtime::stack::StackValue;

use crate::classfile::parse_helper::SafeBuf;

use crate::structs::types::ReferenceType;
use crate::{CallSite, ClassLoader, Vm};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::borrow::BorrowMut;
use std::sync::Arc;

pub fn new(vm: &Vm, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let idx = bytes.try_get_u16()?;

    let entry = ctx.class.const_pool.get(idx.into())?;

    let data = &entry.data.as_class();

    let cls = data.ok_or_else(|| anyhow!("expected class data, got {:?}", entry))?;

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

    let obj = cls.new_instance(vm)?;

    sf.operand_stack
        .push(StackValue::Reference(ReferenceType::Class(obj)));
    Ok(())
}
