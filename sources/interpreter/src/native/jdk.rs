use std::rc::Rc;

use parking_lot::Mutex;
use support::encoding::{encode_string, EncodedString};

use crate::runtime::native::NativeFunction;
use crate::runtime::object::{JavaObject, NameAndDescriptor, RuntimeObject, StringObject};
use crate::runtime::stack::{Array, ArrayType, RuntimeValue};

use crate::{class, instance_method, static_method};

use super::NativeModule;

class!(VM);
class!(Cds);
class!(Unsafe);
class!(Reflection);
class!(SystemPropsRaw);

impl NativeModule for VM {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "initialize", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/VM"
    }
}

impl NativeModule for Unsafe {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
                Ok(None)
            }),
            // TODO: Not a single clue how to implement these
            instance_method!(name: "arrayBaseOffset0", descriptor: "(Ljava/lang/Class;)I" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
            instance_method!(name: "arrayIndexScale0", descriptor: "(Ljava/lang/Class;)I" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
            instance_method!(name: "objectFieldOffset1", descriptor: "(Ljava/lang/Class;Ljava/lang/String;)J" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/Unsafe"
    }
}

impl NativeModule for Cds {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "getRandomSeedForDumping", descriptor: "()J" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
            static_method!(name: "initializeFromArchive", descriptor: "(Ljava/lang/Class;)V" => |_, _, _| {
              Ok(None)
            }),
            static_method!(name: "isDumpingClassList0", descriptor: "()Z" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
            static_method!(name: "isDumpingArchive0", descriptor: "()Z" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
            static_method!(name: "isSharingEnabled0", descriptor: "()Z" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(0.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/CDS"
    }
}

impl NativeModule for SystemPropsRaw {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            // TODO: Implement these
            static_method!(name: "platformProperties", descriptor: "()[Ljava/lang/String;" => |_, _, interpreter| {
                let string_class = interpreter.load_class("java/lang/String".to_string())?;
                let mut values = Vec::with_capacity(100);
                values.resize(100, RuntimeValue::Null);

                let array = Array {
                    ty: ArrayType::Object(string_class),
                    values
                };

                Ok(Some(RuntimeValue::Array(Rc::new(Mutex::new(array)))))
            }),
            static_method!(name: "vmProperties", descriptor: "()[Ljava/lang/String;" => |_, _, interpreter| {
                let string_class = interpreter.load_class("java/lang/String".to_string())?;
                let str = |value: EncodedString| {
                    RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(StringObject::new(Rc::clone(&string_class), value)))))
                };

                let array = Array {
                    ty: ArrayType::Object(Rc::clone(&string_class)),
                    values: vec![
                        str(encode_string("java.home".to_string())?),
                        str(encode_string("nil".to_string())?),
                    ]
                };

                Ok(Some(RuntimeValue::Array(Rc::new(Mutex::new(array)))))
            }),
        ]
    }
    fn classname() -> &'static str {
        "jdk/internal/util/SystemProps$Raw"
    }
}

impl NativeModule for Reflection {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "getCallerClass", descriptor: "()Ljava/lang/Class;" => |_, _, interpreter| {
                // TODO: Implement this with support::unwind
                let java_class = interpreter.load_class("java/lang/Class".to_string())?;
                let string_class = interpreter.load_class("java/lang/String".to_string())?;

                let mut object = RuntimeObject::new(Rc::clone(&java_class));
                object.set_instance_field(
                    ("name".to_string(),"Ljava/lang/String;".to_string()),
                    RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(StringObject::new(
                        string_class,
                        encode_string("CALLER_CLASS".to_string())?,
                    ))))),
                );

                Ok(Some(RuntimeValue::Object(JavaObject::Runtime(Rc::new(Mutex::new(object))))))
            }),
        ]
    }
    fn classname() -> &'static str {
        "jdk/internal/reflect/Reflection"
    }
}
