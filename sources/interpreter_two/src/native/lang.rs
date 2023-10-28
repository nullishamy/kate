use crate::{static_method, instance_method, object::RuntimeValue};

use super::{NativeFunction, NativeModule};

pub struct Class;
pub struct Throwable;

impl NativeModule for Class {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
          static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
            Ok(None)
          }),
          static_method!(name: "desiredAssertionStatus0", descriptor: "(Ljava/lang/Class;)Z" => |_, _, _| {
            Ok(Some(crate::object::RuntimeValue::Integral((1_i32).into())))
          })
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