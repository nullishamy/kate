

use crate::{
    instance_method,
    object::{
        builtins::{Array, ArrayType, BuiltinString, Class, Object, ArrayPrimitive},
        interner::{interner_meta_class, intern_string},
        mem::RefTo,
        runtime::RuntimeValue, layout::types::{Bool, Byte, Short, Double, Float, Char, Int, Long},
    },
    static_method,
};

use super::{NativeFunction, NativeModule};
pub struct Cds;
pub struct Reflection;
pub struct SystemPropsRaw;
pub struct Unsafe;
pub struct JdkVm;

pub struct ScopedMemoryAccess;
pub struct Signal;

impl NativeModule for Cds {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "getRandomSeedForDumping", descriptor: "()J" => |_, _, _| {
                Ok(Some(RuntimeValue::Integral(0_i64.into())))
            }),
            static_method!(name: "initializeFromArchive", descriptor: "(Ljava/lang/Class;)V" => |_, _args, _| {
                Ok(None)
            }),
            static_method!(name: "isDumpingClassList0", descriptor: "()Z" => |_, _, _| {
                Ok(Some(RuntimeValue::Integral(0_i32.into())))
            }),
            static_method!(name: "isDumpingArchive0", descriptor: "()Z" => |_, _, _| {
                Ok(Some(RuntimeValue::Integral(0_i32.into())))
            }),
            static_method!(name: "isSharingEnabled0", descriptor: "()Z" => |_, _, _| {
                Ok(Some(RuntimeValue::Integral(0_i32.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/CDS"
    }
}

impl NativeModule for Reflection {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "getCallerClass", descriptor: "()Ljava/lang/Class;" => |_, _, vm| {
                let mut frames = vm.frames.clone();
                let current_frame = frames.pop().expect("no current frame");
                let current_class = current_frame.class_name;

                let first_frame_that_isnt_ours = frames.into_iter().find(|f| f.class_name != current_class);

                let cls = if let Some(frame) = first_frame_that_isnt_ours {
                    Some(RuntimeValue::Object(vm.class_loader.for_name(frame.class_name)?.erase()))
                } else {
                    None
                };

                Ok(cls)
            }),
        ]
    }

    fn classname() -> &'static str {
        ""
    }
}

impl NativeModule for SystemPropsRaw {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "platformProperties", descriptor: "()[Ljava/lang/String;" => |_, _, _vm| {
                // TODO: Populate these properly

                let mut data = Vec::with_capacity(100);
                data.resize(100, RefTo::null());

                let array: RefTo<Array<RefTo<BuiltinString>>> = Array::from_vec(
                    ArrayType::Object(interner_meta_class()),
                    "Ljava/lang/String;".to_string(),
                    data
                );

                Ok(Some(RuntimeValue::Object(array.erase())))
            }),
            static_method!(name: "vmProperties", descriptor: "()[Ljava/lang/String;" => |_, _, _| {
                // TODO: Populate these properly

                let array: RefTo<Array<RefTo<BuiltinString>>> = Array::from_vec(
                    ArrayType::Object(interner_meta_class()),
                    "Ljava/lang/String;".to_string(),
                    vec![
                        intern_string("java.home".to_string())?,
                        intern_string("nil".to_string())?,

                        intern_string("native.encoding".to_string())?,
                        intern_string("UTF-8".to_string())?,
                        
                        // FIXME: Load from platformProperties
                        intern_string("user.home".to_string())?,
                        intern_string("nil".to_string())?,

                        intern_string("user.dir".to_string())?,
                        intern_string("nil".to_string())?,

                        intern_string("user.name".to_string())?,
                        intern_string("amy b)".to_string())?,

                        intern_string("java.io.tmpdir".to_string())?,
                        intern_string("nil".to_string())?,

                        // FIXME: Not sure where these come from but they need set
                        intern_string("sun.stdout.encoding".to_string())?,
                        intern_string("UTF-8".to_string())?,

                        intern_string("sun.stderr.encoding".to_string())?,
                        intern_string("UTF-8".to_string())?,
                        RefTo::null()
                    ]
                );

                Ok(Some(RuntimeValue::Object(array.erase())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/util/SystemProps$Raw"
    }
}

impl NativeModule for Unsafe {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_, _, _| {
                Ok(None)
            }),
            instance_method!(name: "arrayBaseOffset0", descriptor: "(Ljava/lang/Class;)I" => |_, args, _| {
                let cls = args.get(1).unwrap();
                let cls = cls.as_object().unwrap();
                let cls = unsafe { cls.cast::<Class>() };

                let component = cls.borrow().component_type().unwrap();
                let res = match component {
                    ArrayType::Object(_) => Array::<RefTo<Object>>::elements_offset(),
                    ArrayType::Primitive(ty) => match ty {
                        ArrayPrimitive::Bool => Array::<Bool>::elements_offset(),
                        ArrayPrimitive::Char => Array::<Char>::elements_offset(),
                        ArrayPrimitive::Float => Array::<Float>::elements_offset(),
                        ArrayPrimitive::Double => Array::<Double>::elements_offset(),
                        ArrayPrimitive::Byte => Array::<Byte>::elements_offset(),
                        ArrayPrimitive::Short => Array::<Short>::elements_offset(),
                        ArrayPrimitive::Int => Array::<Int>::elements_offset(),
                        ArrayPrimitive::Long => Array::<Long>::elements_offset(),
                    },
                };

                Ok(Some(RuntimeValue::Integral((res as i32).into())))
            }),
            instance_method!(name: "arrayIndexScale0", descriptor: "(Ljava/lang/Class;)I" => |_, args, _| {
                let cls = args.get(1).unwrap();
                let cls = cls.as_object().unwrap();
                let cls = unsafe { cls.cast::<Class>() };

                let component = cls.borrow().component_type().unwrap();
                let res = match component {
                    ArrayType::Object(_) => Array::<RefTo<Object>>::element_scale(),
                    ArrayType::Primitive(ty) => match ty {
                        ArrayPrimitive::Bool => Array::<Bool>::element_scale(),
                        ArrayPrimitive::Char => Array::<Char>::element_scale(),
                        ArrayPrimitive::Float => Array::<Float>::element_scale(),
                        ArrayPrimitive::Double => Array::<Double>::element_scale(),
                        ArrayPrimitive::Byte => Array::<Byte>::element_scale(),
                        ArrayPrimitive::Short => Array::<Short>::element_scale(),
                        ArrayPrimitive::Int => Array::<Int>::element_scale(),
                        ArrayPrimitive::Long => Array::<Long>::element_scale(),
                    },
                };

                Ok(Some(RuntimeValue::Integral((res as i32).into())))
            }),
            instance_method!(name: "objectFieldOffset1", descriptor: "(Ljava/lang/Class;Ljava/lang/String;)J" => |_, args, _| {
                let cls = {
                    let val = args.get(1).unwrap();
                    let val = val.as_object().unwrap();
                    unsafe { val.cast::<Class>() }
                };

                let field = {
                    let val = args.get(2).unwrap();
                    let val = val.as_object().unwrap();
                    unsafe { val.cast::<BuiltinString>() }
                };

                let layout = cls.borrow().instance_layout();
                let info = layout.field_info(&field.borrow().string()?).expect("TODO: internal error");
                let offset = info.location.offset as i64;

                Ok(Some(RuntimeValue::Integral(offset.into())))
            }),
            instance_method!(name: "storeFence", descriptor: "()V" => |_, _, _| {
                // unimplemented!();
                Ok(None)
            }),
            instance_method!(name: "compareAndSetInt", descriptor: "(Ljava/lang/Object;JII)Z" => |_, args, _| {
                let object = {
                    let val = args.get(1).unwrap();
                    val.as_object().unwrap()
                };

                let offset = {
                    let val = args.get(2).unwrap();
                    val.as_integral().unwrap().value
                };

                let expected = {
                    let val = args.get(4).unwrap();
                    val.as_integral().unwrap().value as i32
                };

                let desired = {
                    let val = args.get(5).unwrap();
                    val.as_integral().unwrap().value as i32
                };

                let raw_ptr = object.borrow_mut() as *mut Object;
                let raw_ptr = unsafe { raw_ptr.byte_add(offset as usize) };
                let raw_ptr = raw_ptr.cast::<i32>();

                // TODO: Make this atomic when we do MT
                let success = {
                    let current = unsafe { raw_ptr.read() };
                    if current == expected {
                        unsafe { raw_ptr.write(desired) };
                        true
                    } else {
                        false
                    }
                };

                Ok(Some(RuntimeValue::Integral((success as i32).into())))
            }),
            instance_method!(name: "compareAndSetReference", descriptor: "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z" => |_, args, _| {
                let object = {
                    let val = args.get(1).unwrap();
                    val.as_object().unwrap()
                };

                let offset = {
                    let val = args.get(2).unwrap();
                    val.as_integral().unwrap().value
                };

                let expected = {
                    let val = args.get(4).unwrap();
                    val.as_object().unwrap()
                };

                let desired = {
                    let val = args.get(5).unwrap();
                    val.as_object().unwrap()
                };

                let raw_ptr = object.borrow_mut() as *mut Object;
                let raw_ptr = unsafe { raw_ptr.byte_add(offset as usize) };
                let raw_ptr = raw_ptr.cast::<RefTo<Object>>();

                // TODO: Make this atomic when we do MT
                let success = {
                    let current = unsafe { raw_ptr.read() };
                    if current.as_ptr() == expected.as_ptr() {
                        unsafe { raw_ptr.write(desired.clone()) };
                        true
                    } else {
                        false
                    }
                };
                Ok(Some(RuntimeValue::Integral((success as i32).into())))
            }),
            instance_method!(name: "compareAndSetLong", descriptor: "(Ljava/lang/Object;JJJ)Z" => |_, args, _| {
                let object = {
                    let val = args.get(1).unwrap();
                    val.as_object().unwrap()
                };

                let offset = {
                    let val = args.get(2).unwrap();
                    val.as_integral().unwrap().value
                };

                let expected = {
                    let val = args.get(4).unwrap();
                    val.as_integral().unwrap().value
                };

                let desired = {
                    let val = args.get(5).unwrap();
                    val.as_integral().unwrap().value
                };

                let raw_ptr = object.borrow_mut() as *mut Object;
                let raw_ptr = unsafe { raw_ptr.byte_add(offset as usize) };
                let raw_ptr = raw_ptr.cast::<i64>();

                // TODO: Make this atomic when we do MT
                let success = {
                    let current = unsafe { raw_ptr.read() };
                    if current == expected {
                        unsafe { raw_ptr.write(desired) };
                        true
                    } else {
                        false
                    }
                };

                Ok(Some(RuntimeValue::Integral((success as i32).into())))
            }),
            instance_method!(name: "getReferenceVolatile", descriptor: "(Ljava/lang/Object;J)Ljava/lang/Object;" => |_, args, _| {
                let object = {
                    let val = args.get(1).unwrap();
                    val.as_object().unwrap()
                };

                let offset = {
                    let val = args.get(2).unwrap();
                    val.as_integral().unwrap().value
                };

                let raw_ptr = object.borrow_mut() as *mut Object;
                let raw_ptr = unsafe { raw_ptr.byte_add(offset as usize) };
                let raw_ptr = raw_ptr.cast::<RefTo<Object>>();
                let val = unsafe { raw_ptr.as_ref().unwrap() }.clone();

                Ok(Some(RuntimeValue::Object(val)))
            }),
            instance_method!(name: "getIntVolatile", descriptor: "(Ljava/lang/Object;J)I" => |_, args, _| {
                let object = {
                    let val = args.get(1).unwrap();
                    val.as_object().unwrap()
                };

                let offset = {
                    let val = args.get(2).unwrap();
                    val.as_integral().unwrap().value
                };

                let raw_ptr = object.borrow_mut() as *mut Object;
                let raw_ptr = unsafe { raw_ptr.byte_add(offset as usize) };
                let raw_ptr = raw_ptr.cast::<Int>();
                let val = unsafe { raw_ptr.read() };

                Ok(Some(RuntimeValue::Integral(val.into())))
            }),
            instance_method!(name: "putReferenceVolatile", descriptor: "(Ljava/lang/Object;JLjava/lang/Object;)V" => |_, args, _| {
                let object = {
                    let val = args.get(1).unwrap();
                    val.as_object().unwrap()
                };

                let offset = {
                    let val = args.get(2).unwrap();
                    val.as_integral().unwrap().value
                };

                let value = {
                    let val = args.get(4).unwrap();
                    val.as_object().unwrap()
                };

                let raw_ptr = object.borrow_mut() as *mut Object;
                let raw_ptr = unsafe { raw_ptr.byte_add(offset as usize) };
                let raw_ptr = raw_ptr.cast::<RefTo<Object>>();
                unsafe { raw_ptr.write(value.clone()) };

                Ok(None)
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/Unsafe"
    }
}

impl NativeModule for JdkVm {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "initialize", descriptor: "()V" => |_cls, _, _| {
                Ok(None)
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/VM"
    }
}

impl NativeModule for ScopedMemoryAccess {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "registerNatives", descriptor: "()V" => |_cls, _, _| {
                Ok(None)
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/ScopedMemoryAccess"
    }
}

impl NativeModule for Signal {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "findSignal0", descriptor: "(Ljava/lang/String;)I" => |_, args, _| {
                let sig = args.get(0).unwrap();
                let sig = sig.as_object().unwrap();
                let sig = unsafe { sig.cast::<BuiltinString>() };
                let sig = sig.borrow().string()?;

                // TODO: Get actual signals
                let code: i32 = match sig.as_str() {
                    "ABRT" => 0, 
                    "FPE" => 1,   
                    "ILL" => 2, 
                    "INT" => 3,
                    "SEGV" => 4, 
                    "TERM" => 5, 
                    "HUP" => 6,
                    _ => -1
                };

                Ok(Some(RuntimeValue::Integral(code.into())))
            }),
            static_method!(name: "handle0", descriptor: "(IJ)J" => |_, _, _| {
                // TODO:
                Ok(Some(RuntimeValue::Integral(0_i64.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/Signal"
    }
}