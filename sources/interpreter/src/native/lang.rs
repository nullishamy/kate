use std::time::SystemTime;

use support::encoding::{decode_string, CompactEncoding};

use crate::{
    instance_method,
    object::{
        builtins::{BuiltinString, Class, Object, Array, ArrayType, ArrayPrimitive},
        interner::intern_string,
        layout::{types::{self, Bool, Byte, Char, Double}, ClassFileLayout},
        mem::RefTo,
        runtime::RuntimeValue,
    },
    static_method, internal,
};

use super::{NativeFunction, NativeModule};

pub struct LangClass;
pub struct Throwable;
pub struct StringUTF16;
pub struct Float;
pub struct LangDouble;
pub struct System;
pub struct LangObject;
pub struct LangThread;
pub struct Runtime;

impl NativeModule for LangThread {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_cls, _, _| {
                Ok(None)
            }),
            static_method!(name: "currentThread", descriptor: "()Ljava/lang/Thread;" => |_, _, vm| {
                Ok(Some(RuntimeValue::Object(vm.main_thread())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Thread"
    }

}

impl NativeModule for System {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_cls, _, _| {
                Ok(None)
            }),
            static_method!(name: "identityHashCode", descriptor: "(Ljava/lang/Object;)I" => |_, args, _| {
                let obj = args.get(0).unwrap();
                let hash = obj.hash_code();

                Ok(Some(RuntimeValue::Integral(hash.into())))
            }),
            static_method!(name: "nanoTime", descriptor: "()J" => |_, _, _| {
                let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                let timestamp_nanos = duration_since_epoch.as_nanos() as u64 as i64;
                Ok(Some(RuntimeValue::Integral(timestamp_nanos.into())))
            }),
            static_method!(name: "setIn0", descriptor: "(Ljava/io/InputStream;)V" => |cls, args, _| {
                let stream = args.get(0).unwrap();
                let stream = stream.as_object().unwrap();
                let field = cls
                    .borrow_mut()
                    .static_field_info_mut((
                        "in".to_string(),
                        "Ljava/io/InputStream;".to_string()
                    ))
                    .unwrap();

                field.value = Some(RuntimeValue::Object(stream.clone()));
                
                Ok(None)
            }),
            static_method!(name: "setOut0", descriptor: "(Ljava/io/PrintStream;)V" => |cls, args, _| {
                let stream = args.get(0).unwrap();
                let stream = stream.as_object().unwrap();
                let field = cls
                    .borrow_mut()
                    .static_field_info_mut((
                        "out".to_string(),
                        "Ljava/io/InputStream;".to_string()
                    ))
                    .unwrap();

                field.value = Some(RuntimeValue::Object(stream.clone()));
                
                Ok(None)
            }),
            static_method!(name: "setErr0", descriptor: "(Ljava/io/PrintStream;)V" => |cls, args, _| {
                let stream = args.get(0).unwrap();
                let stream = stream.as_object().unwrap();
                let field = cls
                    .borrow_mut()
                    .static_field_info_mut((
                        "err".to_string(),
                        "Ljava/io/InputStream;".to_string()
                    ))
                    .unwrap();

                field.value = Some(RuntimeValue::Object(stream.clone()));
                
                Ok(None)
            }),
            static_method!(name: "arraycopy", descriptor: "(Ljava/lang/Object;ILjava/lang/Object;II)V" => |_, args, _| {
                let (src, src_pos, dest, dest_pos, len) = (
                    args.get(0).expect("no arg 0 (src)").as_object().expect("not an object"),
                    args.get(1).expect("no arg 1 (src_pos)").as_integral().expect("not an integral"),
                    args.get(2).expect("no arg 2 (dest)").as_object().expect("not an object"),
                    args.get(3).expect("no arg 3 (dest_pos)").as_integral().expect("not an integral"),
                    args.get(4).expect("no arg 4 (len)").as_integral().expect("not an integral"),
                );

                let src_pos = src_pos.value;
                let dest_pos = dest_pos.value;
                let len = len.value;

                let src_class = src.borrow().class();
                let src_ty = src_class.borrow();
                
                let dest_class = dest.borrow().class();
                let dest_ty = dest_class.borrow();
                
                let src_component = src_ty.component_type().unwrap();
                let dest_component = dest_ty.component_type().unwrap();

                if src_component != dest_component {
                    panic!("array store exception")
                }

                if src_pos < 0 {
                    panic!("out of bounds")
                }

                if dest_pos < 0 {
                    panic!("out of bounds")
                }

                if len < 0 {
                    panic!("out of bounds")
                }

                let src_pos = src_pos as usize;
                let dest_pos = dest_pos as usize;
                let len = len as usize;

                match src_component {
                    ArrayType::Object(_) => todo!(),
                    ArrayType::Primitive(ty) => match ty {
                        ArrayPrimitive::Bool => {
                            let src = unsafe { src.cast::<Array<Bool>>() };
                            let src_slice = src.borrow_mut().slice_mut();

                            let dest = unsafe { dest.cast::<Array<Bool>>() };
                            let dest_slice = dest.borrow_mut().slice_mut();
                            dest_slice[dest_pos..dest_pos + len].copy_from_slice(&src_slice[src_pos..src_pos + len]);
                        },
                        ArrayPrimitive::Char => {
                            let src = unsafe { src.cast::<Array<Char>>() };
                            let src_slice = src.borrow_mut().slice_mut();

                            let dest = unsafe { dest.cast::<Array<Char>>() };
                            let dest_slice = dest.borrow_mut().slice_mut();
                            dest_slice[dest_pos..dest_pos + len].copy_from_slice(&src_slice[src_pos..src_pos + len]);
                        },
                        ArrayPrimitive::Float => todo!(),
                        ArrayPrimitive::Double => {
                            let src = unsafe { src.cast::<Array<Double>>() };
                            let src_slice = src.borrow_mut().slice_mut();

                            let dest = unsafe { dest.cast::<Array<Double>>() };
                            let dest_slice = dest.borrow_mut().slice_mut();
                            dest_slice[dest_pos..dest_pos + len].copy_from_slice(&src_slice[src_pos..src_pos + len]);
                        },
                        ArrayPrimitive::Byte => {
                            let src = unsafe { src.cast::<Array<Byte>>() };
                            let src_slice = src.borrow_mut().slice_mut();

                            let dest = unsafe { dest.cast::<Array<Byte>>() };
                            let dest_slice = dest.borrow_mut().slice_mut();
                            dest_slice[dest_pos..dest_pos + len].copy_from_slice(&src_slice[src_pos..src_pos + len]);
                        },
                        ArrayPrimitive::Short => todo!(),
                        ArrayPrimitive::Int => todo!(),
                        ArrayPrimitive::Long => todo!(),
                    },
                }

                Ok(None)
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/System"
    }
}

impl NativeModule for Runtime {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            instance_method!(name: "availableProcessors", descriptor: "()I" => |_, _, _| {
                // TODO: Support MT and report this accurately
                Ok(Some(RuntimeValue::Integral(1_i32.into())))
            }),
            instance_method!(name: "maxMemory", descriptor: "()J" => |_, _, _| {
                // TODO: Read this properly
                Ok(Some(RuntimeValue::Integral(1024_i64.into())))
            }),
            instance_method!(name: "gc", descriptor: "()V" => |_, _, _| {
                // Our impl does not GC!
                Ok(None)
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/Runtime"
    }
}

impl NativeModule for LangClass {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
                Ok(None)
            }),
            instance_method!(name: "isArray", descriptor: "()Z" => |this, _, _vm| {
                let res = if this.borrow().class().borrow().is_array() {
                    1_i32
                } else {
                    0_i32
                };

                Ok(Some(RuntimeValue::Integral(res.into())))
            }),
            instance_method!(name: "isInterface", descriptor: "()Z" => |this, _, _vm| {
                let res = if this.borrow().class().borrow().is_interface() {
                    1_i32
                } else {
                    0_i32
                };

                Ok(Some(RuntimeValue::Integral(res.into())))
            }),
            instance_method!(name: "isPrimitive", descriptor: "()Z" => |this, _, _| {
                let class = unsafe { this.cast::<Class>() };

                let prim = if class.borrow().is_primitive() {
                    1_i32
                } else {
                    0_i32
                };
                Ok(Some(RuntimeValue::Integral(prim.into())))
            }),
            instance_method!(name: "initClassName", descriptor: "()Ljava/lang/String;" => |this, _, _| {
                let class = unsafe { this.cast::<Class>() };

                let name = class.borrow().name();

                Ok(Some(RuntimeValue::Object(intern_string(name.clone())?.erase())))
            }),
            static_method!(name: "desiredAssertionStatus0", descriptor: "(Ljava/lang/Class;)Z" => |_, _, _| {
                Ok(Some(RuntimeValue::Integral((1_i32).into())))
            }),
            static_method!(name: "getPrimitiveClass", descriptor: "(Ljava/lang/String;)Ljava/lang/Class;" => |_, args, vm| {
                let prim_name = args.get(0).cloned().expect("no prim name");
                let prim_name = prim_name.as_object().cloned().expect("arg0 was not an object");

                let prim_name = unsafe {
                    prim_name.cast::<BuiltinString>()
                };

                let bytes = prim_name.borrow().value.borrow().slice().to_vec();
                let prim_str = decode_string((CompactEncoding::from_coder(prim_name.borrow().coder), bytes))?;
                let jlc = vm.class_loader.for_name("java/lang/Class".to_string())?;
                let jlo = vm.class_loader.for_name("java/lang/Object".to_string())?;

                let layout = match prim_str.as_str() {
                    "byte" => ClassFileLayout::from_java_type(types::BYTE),
                    "float" => ClassFileLayout::from_java_type(types::FLOAT),
                    "double" => ClassFileLayout::from_java_type(types::DOUBLE),
                    "int" => ClassFileLayout::from_java_type(types::INT),
                    "char" => ClassFileLayout::from_java_type(types::CHAR),
                    "long" => ClassFileLayout::from_java_type(types::LONG),
                    "boolean" => ClassFileLayout::from_java_type(types::BOOL),
                    p => panic!("unknown primitive {}", p)
                };

                let cls = RefTo::new(Class::new_primitive(
                    Object {
                        class: jlc,
                        super_class: jlo,
                        ref_count: 0,
                    },
                    prim_str,
                    layout
                ));

                Ok(Some(RuntimeValue::Object(cls.erase())))
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
                // let big_endian = if cfg!(target_endian = "big") {
                //     1
                // } else {
                //     0
                // };

                // FIXME: Figure out why setting little endian makes everything explode with stringutf16
                let big_endian = 1;

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
              let float = args.get(0).ok_or(internal!("no arg 0"))?;
              let float = float.as_floating().ok_or(internal!("not a float"))?;

              let result = (float.value as f32).to_bits();
              Ok(Some(RuntimeValue::Integral((result as i32).into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Float"
    }
}

impl NativeModule for LangDouble {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "doubleToRawLongBits", descriptor: "(D)J" => |_cls, args, _| {
              let float = args.get(0).ok_or(internal!("no arg 0"))?;
              let float = float.as_floating().ok_or(internal!("not a float"))?;

              let result = float.value.to_bits() as i64;
              Ok(Some(RuntimeValue::Integral(result.into())))
            }),
            static_method!(name: "longBitsToDouble", descriptor: "(J)D" => |_cls, args, _| {
              let long = args.get(0).ok_or(internal!("no arg 0"))?;
              let long = long.as_integral().ok_or(internal!("not an int"))?;

              let result = long.value as f64;
              Ok(Some(RuntimeValue::Floating(result.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/lang/Double"
    }
}

impl NativeModule for LangObject {
    fn methods() -> Vec<(super::NameAndDescriptor, super::NativeFunction)> {
        vec![
            instance_method!(name: "hashCode", descriptor: "()I" => |this, _, _| {
                let rtv = RuntimeValue::Object(this);
                let hash: i32 = rtv.hash_code();

                Ok(Some(RuntimeValue::Integral(hash.into())))
            }),
            instance_method!(name: "getClass", descriptor: "()Ljava/lang/Class;" => |this, _, _vm| {
                Ok(Some(RuntimeValue::Object(this.borrow().class().erase())))
            }),
            instance_method!(name: "notifyAll", descriptor: "()V" => |_, _, _| {
                // TODO:
                Ok(None)
            }),
        ]
    }
    fn classname() -> &'static str {
        "java/lang/Object"
    }
}
