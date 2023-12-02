use std::{collections::HashMap, fs::File, io::Write, os::fd::FromRawFd};

use crate::{
    error::Throwable,
    instance_method, module_base,
    object::{
        builtins::{Array, Class, Object},
        layout::types::Int,
        mem::{FieldRef, RefTo},
        numeric::FALSE,
        runtime::RuntimeValue,
    },
    static_method, VM,
};

use super::{NameAndDescriptor, NativeFunction, NativeModule};

module_base!(IOFileDescriptor);
impl NativeModule for IOFileDescriptor {
    fn classname(&self) -> &'static str {
        "java/io/FileDescriptor"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn init_ids(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(None)
        }

        self.set_method("initIDs", "()V", static_method!(init_ids));

        fn get_handle(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            // Noop on Unix, would return handle on Windows.
            Ok(Some(RuntimeValue::Integral((-1_i64).into())))
        }

        self.set_method("getHandle", "(I)J", static_method!(get_handle));

        fn get_append(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            // TODO: Figure this one out
            Ok(Some(RuntimeValue::Integral(FALSE)))
        }

        self.set_method("getAppend", "(I)Z", static_method!(get_append));
    }
}

module_base!(IOFileOutputStream);
impl NativeModule for IOFileOutputStream {
    fn classname(&self) -> &'static str {
        "java/io/FileOutputStream"
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }

    fn init(&mut self) {
        fn init_ids(
            _: RefTo<Class>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            Ok(None)
        }

        self.set_method("initIDs", "()V", static_method!(init_ids));

        fn write_bytes(
            this: RefTo<Object>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let fd = {
                let field: FieldRef<RefTo<Object>> = this
                    .borrow_mut()
                    .field(("fd".to_string(), "Ljava/io/FileDescriptor;".to_string()))
                    .unwrap();

                let fd_obj = field.to_ref();
                let fd_int: FieldRef<Int> = fd_obj
                    .borrow_mut()
                    .field(("fd".to_string(), "I".to_string()))
                    .unwrap();

                fd_int.copy_out()
            };

            let bytes = {
                let val = args.get(1).unwrap();
                let val = val.as_object().unwrap();

                unsafe { val.cast::<Array<u8>>() }
            };

            let offset = {
                let val = args.get(2).unwrap();
                let val = val.as_integral().unwrap();
                val.value as i32
            };

            let length = {
                let val = args.get(3).unwrap();
                let val = val.as_integral().unwrap();
                val.value as i32
            };

            let _append = {
                let val = args.get(4).unwrap();
                let val = val.as_integral().unwrap();
                val.value as i32 != 0
            };

            let mut file = unsafe { File::from_raw_fd(fd) };
            let data_ptr = bytes.to_ref().data_ptr();
            let data_start = unsafe { data_ptr.byte_add(offset as usize) };
            let buf = unsafe { &*std::ptr::slice_from_raw_parts(data_start, length as usize) };
            file.write_all(buf).unwrap();

            Ok(None)
        }

        self.set_method("writeBytes", "([BIIZ)V", instance_method!(write_bytes));
    }
}