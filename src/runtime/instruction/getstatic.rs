use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::classfile::parse_helper::SafeBuf;


use crate::{CallSite, ClassLoader, VM};

pub fn get_static(vm: &VM, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

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

    let stack = &mut sf.operand_stack;

    // field_obj is trivially cloneable and is essentially
    // an owned reference to a java value or reference
    // it will always refer to the same object or value
    stack.push(field_obj.clone());
    Ok(())
}
