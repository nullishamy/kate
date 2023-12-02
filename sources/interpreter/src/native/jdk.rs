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

/**
 * See: https://github.com/openjdk/jdk/blob/7d4b77ad9ee803d89eab5632f5c65ac843a68b3c/src/java.base/share/classes/jdk/internal/util/SystemProps.java#L217
 *
 * See: https://github.com/openjdk/jdk/blob/7d4b77ad9ee803d89eab5632f5c65ac843a68b3c/src/java.base/share/native/libjava/System.c#L107
 */
mod fields {
    pub const DISPLAY_COUNTRY_NDX: usize = 0;
    pub const DISPLAY_LANGUAGE_NDX: usize = 1 + DISPLAY_COUNTRY_NDX;
    pub const DISPLAY_SCRIPT_NDX: usize = 1 + DISPLAY_LANGUAGE_NDX;
    pub const DISPLAY_VARIANT_NDX: usize = 1 + DISPLAY_SCRIPT_NDX;

    pub const FILE_ENCODING_NDX: usize = 1 + DISPLAY_VARIANT_NDX;
    pub const FILE_SEPARATOR_NDX: usize = 1 + FILE_ENCODING_NDX;

    pub const FORMAT_COUNTRY_NDX: usize = 1 + FILE_SEPARATOR_NDX;
    pub const FORMAT_LANGUAGE_NDX: usize = 1 + FORMAT_COUNTRY_NDX;
    pub const FORMAT_SCRIPT_NDX: usize = 1 + FORMAT_LANGUAGE_NDX;
    pub const FORMAT_VARIANT_NDX: usize = 1 + FORMAT_SCRIPT_NDX;

    pub const FTP_NON_PROXY_HOSTS_NDX: usize = 1 + FORMAT_VARIANT_NDX;
    pub const FTP_PROXY_HOST_NDX: usize = 1 + FTP_NON_PROXY_HOSTS_NDX;
    pub const FTP_PROXY_PORT_NDX: usize = 1 + FTP_PROXY_HOST_NDX;

    pub const HTTP_NON_PROXY_HOSTS_NDX: usize = 1 + FTP_PROXY_PORT_NDX;
    pub const HTTP_PROXY_HOST_NDX: usize = 1 + HTTP_NON_PROXY_HOSTS_NDX;
    pub const HTTP_PROXY_PORT_NDX: usize = 1 + HTTP_PROXY_HOST_NDX;
    pub const HTTPS_PROXY_HOST_NDX: usize = 1 + HTTP_PROXY_PORT_NDX;
    pub const HTTPS_PROXY_PORT_NDX: usize = 1 + HTTPS_PROXY_HOST_NDX;

    pub const JAVA_IO_TMPDIR_NDX: usize = 1 + HTTPS_PROXY_PORT_NDX;
    pub const LINE_SEPARATOR_NDX: usize = 1 + JAVA_IO_TMPDIR_NDX;

    pub const OS_ARCH_NDX: usize = 1 + LINE_SEPARATOR_NDX;
    pub const OS_NAME_NDX: usize = 1 + OS_ARCH_NDX;
    pub const OS_VERSION_NDX: usize = 1 + OS_NAME_NDX;

    pub const PATH_SEPARATOR_NDX: usize = 1 + OS_VERSION_NDX;

    pub const SOCKS_NON_PROXY_HOSTS_NDX: usize = 1 + PATH_SEPARATOR_NDX;
    pub const SOCKS_PROXY_HOST_NDX: usize = 1 + SOCKS_NON_PROXY_HOSTS_NDX;
    pub const SOCKS_PROXY_PORT_NDX: usize = 1 + SOCKS_PROXY_HOST_NDX;

    pub const SUN_ARCH_ABI_NDX: usize = 1 + SOCKS_PROXY_PORT_NDX;
    pub const SUN_ARCH_DATA_MODEL_NDX: usize = 1 + SUN_ARCH_ABI_NDX;
    pub const SUN_CPU_ENDIAN_NDX: usize = 1 + SUN_ARCH_DATA_MODEL_NDX;
    pub const SUN_CPU_ISALIST_NDX: usize = 1 + SUN_CPU_ENDIAN_NDX;
    pub const SUN_IO_UNICODE_ENCODING_NDX: usize = 1 + SUN_CPU_ISALIST_NDX;
    pub const SUN_JNU_ENCODING_NDX: usize = 1 + SUN_IO_UNICODE_ENCODING_NDX;
    pub const SUN_OS_PATCH_LEVEL_NDX: usize = 1 + SUN_JNU_ENCODING_NDX;
    pub const SUN_STDERR_ENCODING_NDX: usize = 1 + SUN_OS_PATCH_LEVEL_NDX;
    pub const SUN_STDOUT_ENCODING_NDX: usize = 1 + SUN_STDERR_ENCODING_NDX;

    pub const USER_DIR_NDX: usize = 1 + SUN_STDOUT_ENCODING_NDX;
    pub const USER_HOME_NDX: usize = 1 + USER_DIR_NDX;
    pub const USER_NAME_NDX: usize = 1 + USER_HOME_NDX;

    pub const FIXED_LENGTH: usize = 1 + USER_NAME_NDX;
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
                    intern_string("unknown".to_string())?,

                    intern_string("native.encoding".to_string())?,
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
            let mut arr = Vec::with_capacity(fields::FIXED_LENGTH);
            arr.resize(fields::FIXED_LENGTH, RefTo::null());

            // TODO: Ask OS for temp file
            arr[fields::JAVA_IO_TMPDIR_NDX] = intern_string("/tmp/javaio.tmp".to_string())?;

            // TODO: Make this platform specific
            arr[fields::LINE_SEPARATOR_NDX] = intern_string("\n".to_string())?;
            arr[fields::PATH_SEPARATOR_NDX] = intern_string(":".to_string())?;
            arr[fields::FILE_SEPARATOR_NDX] = intern_string("/".to_string())?;

            // TODO: Resolve these
            arr[fields::USER_HOME_NDX] = intern_string("~".to_string())?;
            arr[fields::USER_DIR_NDX] = intern_string("~".to_string())?;

            // TODO: Actual username
            arr[fields::USER_NAME_NDX] = intern_string("admin".to_string())?;

            arr[fields::FILE_ENCODING_NDX] = intern_string("UTF-8".to_string())?;

            let array: RefTo<Array<RefTo<BuiltinString>>> = Array::from_vec(
                ArrayType::Object(interner_meta_class()),
                "Ljava/lang/String;".to_string(),
                arr,
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

            let layout = cls.to_ref().instance_layout();
            let info = layout
                .field_info(&field.to_ref().string()?)
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

            let component = cls.to_ref().component_type().unwrap();
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

            let component = cls.to_ref().component_type().unwrap();
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

        self.set_method("handle0", "(IJ)J", static_method!(handle0));

        fn find_signal0(
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let sig = args.get(0).unwrap();
            let sig = sig.as_object().unwrap();
            let sig = unsafe { sig.cast::<BuiltinString>() };
            let sig = sig.to_ref().string()?;

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
