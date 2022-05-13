#![allow(unreachable_code)]

use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::stack::StackValue;

use crate::structs::loaded::constant_pool::Data;
use crate::structs::types::{Float, Int, PrimitiveType, PrimitiveWithValue, ReferenceType};
use crate::{CallSite, ClassLoader, Vm};

pub fn ldc(vm: &Vm, ctx: &mut CallSite, bytes: &mut Bytes) -> Result<()> {
    let mut lock = ctx.thread.call_stack.lock();
    let sf = lock.peek_mut().expect("call stack was empty?");

    let idx = bytes.try_get_u8()?;

    let entry = ctx.class.const_pool.get(idx as usize)?;

    if !entry.tag.loadable() {
        return Err(anyhow!(
            "attempted to load unloadable data {:#?}",
            entry.tag
        ));
    }

    let data = match &entry.data {
        Data::Integer(data) => StackValue::Primitive(PrimitiveWithValue::Int(data.bytes as Int)),
        Data::Float(data) => StackValue::Primitive(PrimitiveWithValue::Float(data.bytes as Float)),
        Data::Long(_data) => todo!("handle longs"),
        Data::Double(_data) => todo!("handle doubles"),
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
                return Err(anyhow!("unimplemented"));
            } else {
                return Err(anyhow!(
                    "char[] constructor for String could not be located "
                ));
            }

            StackValue::Reference(ReferenceType::Class(instance))
        }
        Data::MethodHandle(_) => todo!(),
        Data::MethodType(_) => todo!(),
        Data::Dynamic(_) => todo!(),
        _ => unreachable!(), // if we cant load it, we should never reach here
    };

    let stack = &mut sf.operand_stack;

    stack.push(data);
    Ok(())
}
