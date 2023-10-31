use std::{fs::File, os::fd::FromRawFd, io::Write};




use crate::{
    instance_method,
    object::{
        builtins::{Object, Array},
        layout::types::Int,
        mem::{RefTo, FieldRef},
        runtime::RuntimeValue,
    },
    static_method,
};

use super::{NativeFunction, NativeModule};

pub struct FileDescriptor;
impl NativeModule for FileDescriptor {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "initIDs", descriptor: "()V" => |_, _, _vm| {
                Ok(None)
            }),
            static_method!(name: "getHandle", descriptor: "(I)J" => |_, _, _vm| {
                // Noop on Unix, would return handle on Windows.
                Ok(Some(RuntimeValue::Integral((-1_i64).into())))
            }),
            static_method!(name: "getAppend", descriptor: "(I)Z" => |_, _, _vm| {
                // TODO: Figure this one out
                Ok(Some(RuntimeValue::Integral(0_i32.into())))
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/io/FileDescriptor"
    }
}
pub struct FileOutputStream;
impl NativeModule for FileOutputStream {
    fn methods() -> Vec<(super::NameAndDescriptor, NativeFunction)> {
        vec![
            static_method!(name: "initIDs", descriptor: "()V" => |_, _, _vm| {
                Ok(None)
            }),
            instance_method!(name: "writeBytes", descriptor: "([BIIZ)V" => |this, args, _vm| {
                let fd = {
                    let field: FieldRef<RefTo<Object>> = this
                        .borrow_mut()
                        .field(("fd".to_string(), "Ljava/io/FileDescriptor;".to_string()))
                        .unwrap();

                    let fd_obj = field.borrow();
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
                let data_ptr = bytes.borrow().data_ptr();
                let data_start = unsafe { data_ptr.byte_add(offset as usize) };
                let buf = unsafe { &*std::ptr::slice_from_raw_parts(data_start, length as usize) };
                file.write_all(buf).unwrap();

                Ok(None)
            }),
        ]
    }

    fn classname() -> &'static str {
        "java/io/FileOutputStream"
    }
}