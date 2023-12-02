use std::collections::HashMap;

use crate::{
    error::Throwable,
    module_base,
    object::{
        builtins::{Array, ArrayPrimitive, ArrayType, BuiltinString, Class, Object},
        interner::{intern_string, interner_meta_class},
        layout::types,
        mem::RefTo,
        numeric::FALSE,
        runtime::RuntimeValue,
    },
    static_method, VM,
};

use super::{NameAndDescriptor, NativeFunction, NativeModule};

module_base!(JdkVM);
impl NativeModule for JdkVM {
    fn classname(&self) -> &'static str {
        "jdk/internal/misc/VM"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn initialize(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(None)
        }

        self.set_method("initialize", "()V", static_method!(initialize));
    }
}

module_base!(JdkCDS);
impl NativeModule for JdkCDS {
    fn classname(&self) -> &'static str {
        "jdk/internal/misc/CDS"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn initialize_from_archive(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(Some(RuntimeValue::Integral(0_i64.into())))
        }

        self.set_method(
            "initializeFromArchive",
            "(Ljava/lang/Class;)V",
            static_method!(initialize_from_archive),
        );

        fn get_random_seed_for_dumping(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(Some(RuntimeValue::Integral(0_i64.into())))
        }

        self.set_method(
            "getRandomSeedForDumping",
            "()J",
            static_method!(get_random_seed_for_dumping),
        );

        fn is_dumping_class_list0(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(Some(RuntimeValue::Integral(FALSE)))
        }

        self.set_method(
            "isDumpingClassList0",
            "()Z",
            static_method!(is_dumping_class_list0),
        );

        fn is_dumping_archive0(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(Some(RuntimeValue::Integral(FALSE)))
        }

        self.set_method(
            "isDumpingArchive0",
            "()Z",
            static_method!(is_dumping_archive0),
        );

        fn is_sharing_enabled0(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(Some(RuntimeValue::Integral(FALSE)))
        }

        self.set_method(
            "isSharingEnabled0",
            "()Z",
            static_method!(is_sharing_enabled0),
        );
    }
}

module_base!(JdkReflection);
impl NativeModule for JdkReflection {
    fn classname(&self) -> &'static str {
        "jdk/internal/reflect/Reflection"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn get_caller_class(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            vm: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let mut frames = vm.frames.clone();
            let current_frame = frames.pop().expect("no current frame");
            let current_class = current_frame.class_name;

            let first_frame_that_isnt_ours =
                frames.into_iter().find(|f| f.class_name != current_class);

            let cls = if let Some(frame) = first_frame_that_isnt_ours {
                Some(RuntimeValue::Object(
                    vm.class_loader.for_name(frame.class_name)?.erase(),
                ))
            } else {
                None
            };

            Ok(cls)
        }

        self.set_method(
            "getCallerClass",
            "()Ljava/lang/Class;",
            static_method!(get_caller_class),
        );
    }
}

module_base!(JdkSystemPropsRaw);
impl NativeModule for JdkSystemPropsRaw {
    fn classname(&self) -> &'static str {
        "jdk/internal/util/SystemProps$Raw"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn vm_properties(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            // TODO: Populate these properly

            let array: RefTo<Array<RefTo<BuiltinString>>> = Array::from_vec(
                ArrayType::Object(interner_meta_class()),
                "Ljava/lang/String;".to_string(),
                vec![
                    intern_string("java.home".to_string())?,
                    intern_string("nil".to_string())?,
                    intern_string("native.encoding".to_string())?,
                    intern_string("UTF-8".to_string())?,
                    // FIXME: Should be set by the JDK, from the platform properties
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
                    RefTo::null(),
                ],
            );

            Ok(Some(RuntimeValue::Object(array.erase())))
        }

        self.set_method(
            "vmProperties",
            "()[Ljava/lang/String;",
            static_method!(vm_properties),
        );

        fn platform_properties(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            // TODO: Populate these properly

            let mut data = Vec::with_capacity(100);
            data.resize(100, RefTo::null());

            let array: RefTo<Array<RefTo<BuiltinString>>> = Array::from_vec(
                ArrayType::Object(interner_meta_class()),
                "Ljava/lang/String;".to_string(),
                data,
            );

            Ok(Some(RuntimeValue::Object(array.erase())))
        }

        self.set_method(
            "platformProperties",
            "()[Ljava/lang/String;",
            static_method!(platform_properties),
        );
    }
}
module_base!(JdkUnsafe);
impl NativeModule for JdkUnsafe {
    fn classname(&self) -> &'static str {
        "jdk/internal/misc/Unsafe"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn object_field_offset1(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
            let info = layout
                .field_info(&field.borrow().string()?)
                .expect("TODO: internal error");
            let offset = info.location.offset as i64;

            Ok(Some(RuntimeValue::Integral(offset.into())))
        }

        self.set_method(
            "objectFieldOffset1",
            "(Ljava/lang/Class;Ljava/lang/String;)J",
            static_method!(object_field_offset1),
        );

        fn register_natives(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(None)
        }

        self.set_method("registerNatives", "()V", static_method!(register_natives));

        fn store_fence(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(None)
        }

        self.set_method("storeFence", "()V", static_method!(store_fence));

        fn compare_and_set_int(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
        }

        self.set_method(
            "compareAndSetInt",
            "(Ljava/lang/Object;JII)Z",
            static_method!(compare_and_set_int),
        );

        fn compare_and_set_reference(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
        }

        self.set_method(
            "compareAndSetReference",
            "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
            static_method!(compare_and_set_reference),
        );

        fn compare_and_set_long(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let object = {
                let val = args.get(1).unwrap();
                val.as_object().unwrap()
            };

            let offset = {
                let val = args.get(2).unwrap();
                val.as_integral().unwrap().value
            };

            // Careful. Skip a slot. `long`s take up 2.
            let expected = {
                let val = args.get(4).unwrap();
                val.as_integral().unwrap().value
            };

            // Careful. Skip a slot. `long`s take up 2.
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
        }

        self.set_method(
            "compareAndSetLong",
            "(Ljava/lang/Object;JJJ)Z",
            static_method!(compare_and_set_long),
        );

        fn get_reference_volatile(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
        }

        self.set_method(
            "getReferenceVolatile",
            "(Ljava/lang/Object;J)Ljava/lang/Object;",
            static_method!(get_reference_volatile),
        );

        fn get_int_volatile(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
            let raw_ptr = raw_ptr.cast::<types::Int>();
            let val = unsafe { raw_ptr.read() };

            Ok(Some(RuntimeValue::Integral(val.into())))
        }

        self.set_method(
            "getIntVolatile",
            "(Ljava/lang/Object;J)I",
            static_method!(get_int_volatile),
        );

        fn put_reference_volatile(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
        }

        self.set_method(
            "putReferenceVolatile",
            "(Ljava/lang/Object;JLjava/lang/Object;)V",
            static_method!(put_reference_volatile),
        );

        fn array_index_scale0(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            use crate::object::layout::types::*;
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
        }

        self.set_method(
            "arrayIndexScale0",
            "(Ljava/lang/Class;)I",
            static_method!(array_index_scale0),
        );

        fn array_base_offset0(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            use crate::object::layout::types::*;
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
        }

        self.set_method(
            "arrayBaseOffset0",
            "(Ljava/lang/Class;)I",
            static_method!(array_base_offset0),
        );
    }
}

module_base!(JdkScopedMemoryAccess);
impl NativeModule for JdkScopedMemoryAccess {
    fn classname(&self) -> &'static str {
        "jdk/internal/misc/ScopedMemoryAccess"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn register_natives(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(None)
        }

        self.set_method("registerNatives", "()V", static_method!(register_natives));
    }
}

module_base!(JdkSignal);
impl NativeModule for JdkSignal {
    fn classname(&self) -> &'static str {
        "jdk/internal/misc/Signal"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn handle0(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            // TODO:
            Ok(Some(RuntimeValue::Integral(0_i64.into())))
        }

        self.set_method(
            "handle0",
            "(IJ)J",
            static_method!(handle0),
        );

        fn find_signal0(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
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
                _ => -1,
            };

            Ok(Some(RuntimeValue::Integral(code.into())))
        }

        self.set_method(
            "findSignal0",
            "(Ljava/lang/String;)I",
            static_method!(find_signal0),
        );
    }
}

/*

impl NativeModule for Signal {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
        ]
    }

    fn classname() -> &'static str {
        "jdk/internal/misc/Signal"
    }
}
*/
