use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek, Write},
    os::fd::{AsRawFd, FromRawFd},
};

use parking_lot::Mutex;

use crate::{
    error::Throwable,
    instance_method, module_base,
    object::{
        builtins::{Array, BuiltinString, Class, Object},
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
                    .unwrap_mut()
                    .field(("fd".to_string(), "Ljava/io/FileDescriptor;".to_string()))
                    .unwrap();

                let fd_obj = field.unwrap_ref();
                let fd_int: FieldRef<Int> = fd_obj
                    .unwrap_mut()
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
            let data_ptr = bytes.unwrap_ref().data_ptr();
            let data_start = unsafe { data_ptr.byte_add(offset as usize) };
            let buf = unsafe { &*std::ptr::slice_from_raw_parts(data_start, length as usize) };
            file.write_all(buf).unwrap();

            // Don't auto drop
            std::mem::forget(file);

            Ok(None)
        }

        self.set_method("writeBytes", "([BIIZ)V", instance_method!(write_bytes));
    }
}

module_base!(IOUnixFileSystem);
impl NativeModule for IOUnixFileSystem {
    fn classname(&self) -> &'static str {
        "java/io/UnixFileSystem"
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
    }
}

lazy_static::lazy_static! {
    static ref FILES: Mutex<HashMap<usize, BufReader<File>>> = {
        Mutex::new(HashMap::new())
    };
}

module_base!(IOFileInputStream);
impl NativeModule for IOFileInputStream {
    fn classname(&self) -> &'static str {
        "java/io/FileInputStream"
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

        fn open0(
            this: RefTo<Object>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let fd_ref = {
                let field: FieldRef<RefTo<Object>> = this
                    .unwrap_mut()
                    .field(("fd".to_string(), "Ljava/io/FileDescriptor;".to_string()))
                    .unwrap();

                let fd_obj = field.unwrap_ref();
                let fd_int: FieldRef<Int> = fd_obj
                    .unwrap_mut()
                    .field(("fd".to_string(), "I".to_string()))
                    .unwrap();

                fd_int
            };

            let path = {
                let val = args.get(1).unwrap();
                let val = val.as_object().unwrap();
                unsafe { val.cast::<BuiltinString>() }
            };

            let file = File::open(path.unwrap_ref().string()?).unwrap();
            fd_ref.write(file.as_raw_fd());

            let file = BufReader::new(file);
            let mut files = FILES.lock();
            files.insert(this.as_ptr() as usize, file);

            Ok(None)
        }

        self.set_method("open0", "(Ljava/lang/String;)V", instance_method!(open0));

        fn length0(
            this: RefTo<Object>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let files = FILES.lock();
            let file = files.get(&(this.as_ptr() as usize)).unwrap();
            let meta = file.get_ref().metadata().unwrap();

            Ok(Some(RuntimeValue::Integral((meta.len() as i64).into())))
        }

        self.set_method("length0", "()J", instance_method!(length0));

        fn position0(
            this: RefTo<Object>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let mut files = FILES.lock();
            let file = files.get_mut(&(this.as_ptr() as usize)).unwrap();
            let pos = file.stream_position().unwrap();

            Ok(Some(RuntimeValue::Integral((pos as i64).into())))
        }

        self.set_method("position0", "()J", instance_method!(position0));

        fn read_bytes(
            this: RefTo<Object>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let mut files = FILES.lock();
            let file = files.get_mut(&(this.as_ptr() as usize)).unwrap();

            let output_array = {
                let arr = args.get(1).unwrap().as_object().unwrap();
                unsafe { arr.cast::<Array<u8>>() }
            };

            let offset = {
                let num = args.get(2).unwrap().as_integral().unwrap();
                num.value as i32
            };

            let len = {
                let num = args.get(3).unwrap().as_integral().unwrap();
                num.value as i32
            };

            if len == 0 {
                return Ok(Some(RuntimeValue::Integral(0_i32.into())));
            }

            let subslice = {
                let slice = output_array.unwrap_mut().slice_mut();
                &mut slice[(offset as usize)..(len as usize)]
            };

            let read_n = file.read(subslice).unwrap();

            Ok(Some(RuntimeValue::Integral((read_n as i32).into())))
        }

        self.set_method("readBytes", "([BII)I", instance_method!(read_bytes));

        fn read0(
            this: RefTo<Object>,
            _: Vec<RuntimeValue>,
            _: &mut VM,
        ) -> Result<Option<RuntimeValue>, Throwable> {
            let mut files = FILES.lock();
            let file = files.get_mut(&(this.as_ptr() as usize)).unwrap();

            let mut slice: [u8; 1] = [0];
            let read_n = file.read(&mut slice).unwrap();
            if read_n == 0 {
                // EOF
                Ok(Some(RuntimeValue::Integral((-1_i32).into())))
            } else {
                Ok(Some(RuntimeValue::Integral((slice[0] as i32).into())))
            }

        }

        self.set_method("read0", "()I", instance_method!(read0));
    }
}
