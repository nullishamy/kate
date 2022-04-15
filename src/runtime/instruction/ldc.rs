use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::stack::OperandType;
use crate::structs::loaded::constant_pool::Data;
use crate::structs::types::{Float, Int, PrimitiveType, PrimitiveWithValue};
use crate::{ClassLoader, Context, VM};
use anyhow::{anyhow, Result};
use bytes::Bytes;


pub fn ldc(vm: &mut VM, ctx: &mut Context, bytes: &mut Bytes) -> Result<()> {
    let idx = bytes.try_get_u8()?;
    let entry = ctx.class.const_pool.get(idx as usize)?;

    if !entry.tag.loadable() {
        return Err(anyhow!(
            "attempted to load unloadable data {:#?}",
            entry.tag
        ));
    }

    let data = match &entry.data {
        Data::Integer(data) => OperandType::Primitive(PrimitiveWithValue::Int(data.bytes as Int)),
        Data::Float(data) => OperandType::Primitive(PrimitiveWithValue::Float(data.bytes as Float)),
        Data::Long(_data) => OperandType::Primitive(PrimitiveWithValue::Long(todo!())),
        Data::Double(_data) => OperandType::Primitive(PrimitiveWithValue::Double(todo!())),
        Data::Class(_) => todo!(),
        Data::String(_data) => {
            let mut loader = vm.system_classloader.write();

            let string_class = loader.load_class("java/lang/String")?;
            drop(loader);

            let instance = string_class.new_instance(vm)?;

            let cons = instance.class.constructors.find(|c| {
                c.descriptor
                    .parameters
                    .first()
                    .filter(|f| {
                        f.as_array()
                            .filter(|a| {
                                a._type
                                    .as_primitive()
                                    .filter(|p| **p == PrimitiveType::Char)
                                    .is_some()
                            })
                            .is_some()
                    })
                    .is_some()
            });

            if let Some(_c) = cons {
            } else {
                return Err(anyhow!(
                    "char[] constructor for String could not be located "
                ));
            }

            OperandType::Reference(instance)
        }
        Data::MethodHandle(_) => todo!(),
        Data::MethodType(_) => todo!(),
        Data::Dynamic(_) => todo!(),
        _ => unreachable!(), // if we cant load it, we should never reach here
    };

    let mut stack = ctx.thread.operand_stack.lock();

    stack.push(data);
    Err(anyhow!("not implemented"))
}
