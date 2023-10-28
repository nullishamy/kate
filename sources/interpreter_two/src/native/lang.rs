use std::rc::Rc;

use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use parse::{classfile::Methods, flags::ClassFileAccessFlags, pool::ConstantPool};
use support::encoding::{decode_string, CompactEncoding};
use tracing::info;

use crate::{
    instance_method,
    object::{ClassObject, RuntimeValue},
    static_method,
};

use super::{NativeFunction, NativeModule};

pub struct Class;
pub struct Throwable;
pub struct StringUTF16;
pub struct Float;
pub struct Double;
pub struct System;

impl NativeModule for System {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
            static_method!(name: "arraycopy", descriptor: "(Ljava/lang/Object;ILjava/lang/Object;II)V" => |_, args, _| {
              let (src, src_pos, dest, dest_pos, len) = (
                args.get(0).ok_or(anyhow!("no arg 0 (src)"))?,
                args.get(1).ok_or(anyhow!("no arg 1 (src_pos)"))?,
                args.get(2).ok_or(anyhow!("no arg 2 (dest)"))?,
                args.get(3).ok_or(anyhow!("no arg 3 (dest_pos)"))?,
                args.get(4).ok_or(anyhow!("no arg 4 (len)"))?,
              );

              let src = src.as_array().ok_or(anyhow!("src was not an array"))?.write();
              let src_pos = src_pos.as_integral().ok_or(anyhow!("src_pos was not an integral"))?.value as usize;

              let mut dest = dest.as_array().ok_or(anyhow!("dest was not an array"))?.write();
              let dest_pos = dest_pos.as_integral().ok_or(anyhow!("dest_pos was not an integral"))?.value as usize;

              let len = len.as_integral().ok_or(anyhow!("len was not an integral"))?.value as usize;
              info!("arraycopy: copying from src {} -> {} to dest {} -> {} (len {})", src_pos, src_pos + len - 1, dest_pos, dest_pos + len - 1, len);
              if len == 0 {
                return Ok(None)
              }

              dest.values[dest_pos..dest_pos + len].clone_from_slice(&src.values[src_pos..src_pos + len]);

              Ok(None)
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/System"
    }
}

impl NativeModule for Class {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
            static_method!(name: "desiredAssertionStatus0", descriptor: "(Ljava/lang/Class;)Z" => |_, _, _| {
              Ok(Some(crate::object::RuntimeValue::Integral((1_i32).into())))
            }),
            static_method!(name: "getPrimitiveClass", descriptor: "(Ljava/lang/String;)Ljava/lang/Class;" => |_, args, _| {
              let cls = args.get(0).context("no class")?;
              let message = cls.as_object().context("message was not an object")?;
              let message = message.read();

              let bytes = message
                  .get_instance_field(("value".to_string(), "[B".to_string()))
                  .context("could not locate value field")?;

              let bytes = bytes.as_array().context("bytes was not an array (byte[])")?;
              let bytes = bytes
                  .read()
                  .values
                  .iter()
                  .map(|v| v.as_integral().expect("value was not an int (char)"))
                  .map(|v| v.value as u8)
                  .collect::<Vec<_>>();

              let prim = decode_string((CompactEncoding::Utf16, bytes))?;
              let cls = ClassObject::new(None, None, Methods { values: vec![] }, ConstantPool { entries: Rc::new(RwLock::new(Vec::new())) }, ClassFileAccessFlags::from_bits(0)?, prim)?;
              let cls = Rc::new(RwLock::new(cls));

              Ok(Some(RuntimeValue::Object(cls)))
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/Class"
    }
}

impl NativeModule for Throwable {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            instance_method!(name: "fillInStackTrace", descriptor: "(I)Ljava/lang/Throwable;" => |this, _, _| {
              Ok(Some(RuntimeValue::Object(this)))
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/Throwable"
    }
}
impl NativeModule for StringUTF16 {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "isBigEndian", descriptor: "()Z" => |_, _, _| {
                let big_endian = if cfg!(target_endian = "big") {
                    1
                } else {
                    0
                };

                Ok(Some(RuntimeValue::Integral(big_endian.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/StringUTF16"
    }
}

impl NativeModule for Float {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "floatToRawIntBits", descriptor: "(F)I" => |_cls, args, _| {
              let float = args.get(0).ok_or(anyhow!("no arg 0"))?;
              let float = float.as_floating().ok_or(anyhow!("not a float"))?;

              let result = (float.value as f32).to_bits();
              Ok(Some(RuntimeValue::Integral((result as i32).into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Float"
    }
}

impl NativeModule for Double {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "doubleToRawLongBits", descriptor: "(D)J" => |_cls, args, _| {
              let float = args.get(0).ok_or(anyhow!("no arg 0"))?;
              let float = float.as_floating().ok_or(anyhow!("not a float"))?;

              let result = float.value.to_bits() as i64;
              Ok(Some(RuntimeValue::Integral(result.into())))
            }),
            static_method!(name: "longBitsToDouble", descriptor: "(J)D" => |_cls, args, _| {
              let long = args.get(0).ok_or(anyhow!("no arg 0"))?;
              let long = long.as_integral().ok_or(anyhow!("not an int"))?;

              let result = long.value as f64;
              Ok(Some(RuntimeValue::Floating(result.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Double"
    }
}
