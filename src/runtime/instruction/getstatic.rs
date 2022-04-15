use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::stack::OperandType;
use crate::{ClassLoader, Context, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::rc::Rc;
use std::sync::Arc;

pub fn get_static(vm: &mut VM, ctx: &mut Context, bytes: &mut Bytes) -> Result<()> {
    let idx = bytes.try_get_u16()?;

    let field_ref = ctx.class.const_pool.field(idx as usize)?;
    let class_name = &field_ref.class.name.str;

    let mut loader = vm.system_classloader.write();
    let class = loader.load_class(class_name)?;

    let lock = class.fields.read();
    let field_data = lock.statics.get(&field_ref.name_and_type.name.str);

    if field_data.is_none() {
        return Err(anyhow!(
            "unknown static field {}",
            field_ref.name_and_type.name.str
        ));
    }

    let field_obj = field_data.unwrap();

    let mut stack = ctx.thread.operand_stack.lock();

    stack.push(OperandType::Reference(Arc::clone(field_obj)));
    Ok(())
}
