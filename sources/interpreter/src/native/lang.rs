use std::{process, rc::Rc};

use crate::runtime::{
    object::{JavaObject, NameAndDescriptor, RuntimeObject, StringObject},
    stack::{Integral, RuntimeValue},
};
use anyhow::anyhow;
use parking_lot::Mutex;
use std::time::SystemTime;
use support::encoding::{decode_string, encode_string};

use super::NativeModule;
use crate::{class, instance_method, runtime::native::NativeFunction, static_method};

class!(Runtime);
class!(System);
class!(Shutdown);
class!(Object);
class!(Class);
class!(Throwable);
class!(StringUTF16);
class!(Float);
class!(Double);

impl NativeModule for Runtime {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            instance_method!(name: "availableProcessors", descriptor: "()I" => |_, _, _| {
                // TODO: Support MT and report this accurately
                Ok(Some(RuntimeValue::Integral(1.into())))
            }),
            instance_method!(name: "maxMemory", descriptor: "()J" => |_, _, _| {
                // TODO: Read this properly
                Ok(Some(RuntimeValue::Integral(Integral::long(1024))))
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/Runtime"
    }
}

impl NativeModule for StringUTF16 {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
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

impl NativeModule for System {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
            static_method!(name: "identityHashCode", descriptor: "(Ljava/lang/Object;)I" => |_, args, _| {
              let obj = args.get(0).ok_or(anyhow!("no object"))?.as_object().ok_or(anyhow!("not an object"))?;
              let hash = obj.hash_code();
              Ok(Some(RuntimeValue::Integral(hash.into())))
            }),
            static_method!(name: "setIn0", descriptor: "(Ljava/io/InputStream;)V" => |cls, args, _| {
              let mut cls = cls.lock();
              cls.set_static_field((
                "in".to_string(),
                "Ljava/io/PrintStream;".to_string()
               ), args.get(0).cloned().unwrap()
            );

              Ok(None)
            }),
            static_method!(name: "setOut0", descriptor: "(Ljava/io/InputStream;)V" => |cls, args, _| {
              let mut cls = cls.lock();
              cls.set_static_field((
                "out".to_string(),
                "Ljava/io/PrintStream;".to_string()
               ), args.get(0).cloned().unwrap()
            );

              Ok(None)
            }),
            static_method!(name: "setErr0", descriptor: "(Ljava/io/InputStream;)V" => |cls, args, _| {
              let mut cls = cls.lock();
              cls.set_static_field((
                "err".to_string(),
                "Ljava/io/PrintStream;".to_string()
                ),
                args.get(0).cloned().unwrap()
            );

              Ok(None)
            }),
            static_method!(name: "nanoTime", descriptor: "()J" => |_, _, _| {
              let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
              let timestamp_nanos = duration_since_epoch.as_nanos() as u64 as i64;
              Ok(Some(RuntimeValue::Integral(timestamp_nanos.into())))
            }),
            static_method!(name: "arraycopy", descriptor: "(Ljava/lang/Object;ILjava/lang/Object;II)V" => |_, args, _| {
                let (src, src_pos, dest, dest_pos, len) = (
                    args.get(0).ok_or(anyhow!("no arg 0 (src)"))?,
                    args.get(1).ok_or(anyhow!("no arg 1 (src_pos)"))?,
                    args.get(2).ok_or(anyhow!("no arg 2 (dest)"))?,
                    args.get(3).ok_or(anyhow!("no arg 3 (dest_pos)"))?,
                    args.get(4).ok_or(anyhow!("no arg 4 (len)"))?,
                );

                let src = src.as_array().ok_or(anyhow!("src was not an array"))?.lock();
                let src_pos = src_pos.as_integral().ok_or(anyhow!("src_pos was not an integral"))?.value as usize;

                let mut dest = dest.as_array().ok_or(anyhow!("dest was not an array"))?.lock();
                let dest_pos = dest_pos.as_integral().ok_or(anyhow!("dest_pos was not an integral"))?.value as usize;

                let len = len.as_integral().ok_or(anyhow!("len was not an integral"))?.value as usize;
                // info!("arraycopy: copying from src {} -> {} to dest {} -> {} (len {})", src_pos, src_pos + len - 1, dest_pos, dest_pos + len - 1, len);
                if len == 0 {
                    return Ok(None)
                }

                dest.values[dest_pos..dest_pos + len - 1].clone_from_slice(&src.values[src_pos..src_pos + len - 1]);

                Ok(None)
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/System"
    }
}

impl NativeModule for Shutdown {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "beforeHalt", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
            static_method!(name: "halt0", descriptor: "(I)V" => |_cls, args, _| {
              let exit_code = args.get(0).expect("no arg").as_integral().expect("not int");
              process::exit(exit_code.value as i32);
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Shutdown"
    }
}

impl NativeModule for Object {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            instance_method!(name: "notifyAll", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
            instance_method!(name: "hashCode", descriptor: "()I" => |cls, _args, _| {
              let hash = cls.hash_code();
              Ok(Some(RuntimeValue::Integral(hash.into())))
            }),
            instance_method!(name: "getClass", descriptor: "()Ljava/lang/Class;" => |cls, _args, interpreter| {
                let java_class = interpreter.load_class("java/lang/Class".to_string())?;
                let string_class = interpreter.load_class("java/lang/String".to_string())?;
                let class_name = match cls {
                    JavaObject::Runtime(data) => data.lock().class.lock().get_class_name(),
                    JavaObject::String(data) => data.lock().class.lock().get_class_name(),
                };

                let mut object = RuntimeObject::new(Rc::clone(&java_class));

                object.set_instance_field(
                    ("name".to_string(), "Ljava/lang/String;".to_string()),
                    RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(StringObject::new(
                        string_class,
                        encode_string(class_name)?,
                    ))))),
                );
                
                Ok(Some(RuntimeValue::Object(JavaObject::Runtime(Rc::new(Mutex::new(object))))))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Object"
    }
}

impl NativeModule for Class {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
              Ok(None)
            }),
            static_method!(name: "desiredAssertionStatus0", descriptor: "(Ljava/lang/Class;)Z" => |_, _, _| {
              Ok(Some(RuntimeValue::Integral(1.into())))
            }),
            static_method!(name: "getPrimitiveClass", descriptor: "(Ljava/lang/String;)Ljava/lang/Class;" => |_cls, args, interpreter| {
                let class = args.get(0).ok_or(anyhow!("no arg"))?;
                let class = class.as_object().ok_or(anyhow!("not an object"))?.as_string().ok_or(anyhow!("not a string"))?;
                let class = class.lock();

                let value = decode_string(class.value.clone())?;
                let primitive = match value.as_str() {
                    "float" => interpreter.load_primitive("float".to_string())?,
                    "double" => interpreter.load_primitive("double".to_string())?,
                    "int" => interpreter.load_primitive("int".to_string())?,
                    "byte" => interpreter.load_primitive("byte".to_string())?,
                    "char" => interpreter.load_primitive("char".to_string())?,
                    e => panic!("unknown primitive {}", e)
                };

                Ok(Some(RuntimeValue::Object(JavaObject::Runtime(primitive))))
            }),
            instance_method!(name: "isArray", descriptor: "()Z" => |_, _, _| {
                // TODO: Implement this properly when we have better unification of our native object models
                Ok(Some(RuntimeValue::Integral(0.into())))
            }),
            instance_method!(name: "isPrimitive", descriptor: "()Z" => |cls, _, _| {
                let is_prim = cls.as_runtime().unwrap().lock().is_primitive;
                Ok(Some(RuntimeValue::Integral(if is_prim { 1 } else { 0 }.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Class"
    }
}

impl NativeModule for Throwable {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            instance_method!(name: "fillInStackTrace", descriptor: "(I)Ljava/lang/Throwable;" => |cls, _args, _| {
              Ok(Some(RuntimeValue::Object(cls)))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Throwable"
    }
}

impl NativeModule for Float {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "floatToRawIntBits", descriptor: "(F)I" => |_cls, args, _| {
              let float = args.get(0).ok_or(anyhow!("no arg 0"))?;
              let float = float.as_floating().ok_or(anyhow!("not a float"))?;

              let result = (float.value as f32).to_bits();
              Ok(Some(RuntimeValue::Integral((result as i64).into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Float"
    }
}

impl NativeModule for Double {
    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "doubleToRawLongBits", descriptor: "(D)J" => |_cls, args, _| {
              let float = args.get(0).ok_or(anyhow!("no arg 0"))?;
              let float = float.as_floating().ok_or(anyhow!("not a float"))?;

              let result = float.to_bits() as i64;
              Ok(Some(RuntimeValue::Integral(result.into())))
            }),
            static_method!(name: "longBitsToDouble", descriptor: "(J)D" => |_cls, args, _| {
              let long = args.get(0).ok_or(anyhow!("no arg 0"))?;
              let long = *long.as_integral().ok_or(anyhow!("not an int"))?;

              let result = long.value as f64;
              Ok(Some(RuntimeValue::Floating(result.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Double"
    }
}
