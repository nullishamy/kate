pub type JVMPointer = isize;

pub mod primitive_type {
    use crate::types::JVMPointer;

    pub type Boolean = bool;
    pub type Byte = i8;
    pub type Short = i16;
    pub type Int = i32;
    pub type Long = i64;
    pub type Char = i16;
    pub type Float = f32;
    pub type Double = f64;
    pub type ReturnAddress = JVMPointer;
}

pub mod classfile_type {
    use crate::classfile::invalid_format;
    use bytes::Bytes;
    use std::borrow::{Borrow, BorrowMut};

    pub type Index = u16;

    pub struct ByteCode<'a> {
        data: Bytes,
        file_name: &'a str,
    }

    impl<'a> ByteCode<'a> {
        pub fn new(data: Vec<u8>, file_name: &'a str) -> ByteCode<'a> {
            Self {
                data: Bytes::copy_from_slice(&data),
                file_name,
            }
        }

        pub fn data(&mut self) -> &mut Bytes {
            self.data.borrow_mut()
        }

        pub fn file_name(&self) -> &'a str {
            self.file_name
        }
    }

    pub struct ConstantPoolInfo {
        size: u16,
        data: Vec<ConstantPoolEntry>,
    }

    impl ConstantPoolInfo {
        pub fn new(size: u16) -> Self {
            Self {
                size,
                data: Vec::with_capacity(size as usize),
            }
        }

        pub fn data(&mut self) -> &mut Vec<ConstantPoolEntry> {
            self.data.borrow_mut()
        }

        pub fn size(&self) -> u16 {
            self.size
        }
    }

    pub struct ConstantPoolEntry {
        tag: ConstantPoolTag,
        data: ConstantPoolData,
    }

    impl ConstantPoolEntry {
        pub fn new(tag: ConstantPoolTag, data: ConstantPoolData) -> Self {
            Self { tag, data }
        }
    }

    pub enum ConstantPoolData {
        Utf8 {
            length: u16,
            bytes: Vec<u8>,
            as_str: String,
        },
        Integer {
            bytes: u32,
        },
        Float {
            bytes: f32,
        },
        Long {
            //TODO: this has to take up 2 entries (?)
            low_bytes: u32,
            high_bytes: u32,
        },
        Double {
            //TODO: this has to take up 2 entries (?)
            low_bytes: f32,
            high_bytes: f32,
        },
        Class {
            name_index: u16,
        },
        String {
            utf8_index: u16,
        },
        FieldRef {
            class_index: u16,
            name_and_type_index: u16,
        },
        MethodRef {
            class_index: u16,
            name_and_type_index: u16,
        },
        InterfaceMethodRef {
            class_index: u16,
            name_and_type_index: u16,
        },
        NameAndType {
            name_index: u16,
            descriptor_index: u16,
        },
        MethodHandle {
            reference_kind: u8,
            reference_index: u16,
        },
        MethodType {
            descriptor_index: u16,
        },
        Dynamic {
            bootstrap_method_attr_index: u16,
            name_and_type_index: u16,
        },
        InvokeDynamic {
            bootstrap_method_attr_index: u16,
            name_and_type_index: u16,
        },
        Module {
            name_index: u16,
        },
        Package {
            name_index: u16,
        },
    }

    pub enum ConstantPoolTag {
        Utf8,
        Integer,
        Float,
        Long,
        Double,
        Class,
        String,
        FieldRef,
        MethodRef,
        InterfaceMethodRef,
        NameAndType,
        MethodHandle,
        MethodType,
        Dynamic,
        InvokeDynamic,
        Module,
        Package,
    }

    impl ConstantPoolTag {
        fn tag_value(&self) -> u8 {
            match self {
                ConstantPoolTag::Utf8 => 1,
                ConstantPoolTag::Integer => 3,
                ConstantPoolTag::Float => 4,
                ConstantPoolTag::Long => 5,
                ConstantPoolTag::Double => 6,
                ConstantPoolTag::Class => 7,
                ConstantPoolTag::String => 8,
                ConstantPoolTag::FieldRef => 9,
                ConstantPoolTag::MethodRef => 10,
                ConstantPoolTag::InterfaceMethodRef => 11,
                ConstantPoolTag::NameAndType => 12,
                ConstantPoolTag::MethodHandle => 15,
                ConstantPoolTag::MethodType => 16,
                ConstantPoolTag::Dynamic => 17,
                ConstantPoolTag::InvokeDynamic => 18,
                ConstantPoolTag::Module => 19,
                ConstantPoolTag::Package => 20,
            }
        }

        pub(crate) fn new(tag: u8, byte_code: &ByteCode) -> Self {
            match tag {
                1 => ConstantPoolTag::Utf8,
                3 => ConstantPoolTag::Integer,
                4 => ConstantPoolTag::Float,
                5 => ConstantPoolTag::Long,
                6 => ConstantPoolTag::Double,
                7 => ConstantPoolTag::Class,
                8 => ConstantPoolTag::String,
                9 => ConstantPoolTag::FieldRef,
                10 => ConstantPoolTag::MethodRef,
                11 => ConstantPoolTag::InterfaceMethodRef,
                12 => ConstantPoolTag::NameAndType,
                15 => ConstantPoolTag::MethodHandle,
                16 => ConstantPoolTag::MethodType,
                17 => ConstantPoolTag::Dynamic,
                18 => ConstantPoolTag::InvokeDynamic,
                19 => ConstantPoolTag::Module,
                20 => ConstantPoolTag::Package,
                _ => invalid_format(
                    byte_code.file_name,
                    format!("unknown constant pool tag {}", tag).borrow(),
                ),
            }
        }

        fn loadable(&self) -> bool {
            match self {
                ConstantPoolTag::Utf8 => false,
                ConstantPoolTag::Integer => true,
                ConstantPoolTag::Float => true,
                ConstantPoolTag::Long => true,
                ConstantPoolTag::Double => true,
                ConstantPoolTag::Class => true,
                ConstantPoolTag::String => true,
                ConstantPoolTag::FieldRef => false,
                ConstantPoolTag::MethodRef => false,
                ConstantPoolTag::InterfaceMethodRef => false,
                ConstantPoolTag::NameAndType => false,
                ConstantPoolTag::MethodHandle => true,
                ConstantPoolTag::MethodType => true,
                ConstantPoolTag::Dynamic => true,
                ConstantPoolTag::InvokeDynamic => false,
                ConstantPoolTag::Module => false,
                ConstantPoolTag::Package => false,
            }
        }
    }
    pub struct FieldInfo {
        data: Vec<FieldInfoEntry>,
    }

    impl FieldInfo {
        pub fn new(data: Vec<FieldInfoEntry>) -> Self {
            Self { data }
        }
    }
    pub struct FieldInfoEntry {
        access_flags: u16,
        name_index: u16,
        descriptor_index: u16,
        attributes_count: u16,
        attribute_info: AttributeInfo,
    }

    impl FieldInfoEntry {
        pub fn new(
            access_flags: u16,
            name_index: u16,
            descriptor_index: u16,
            attributes_count: u16,
            attribute_info: AttributeInfo,
        ) -> Self {
            Self {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attribute_info,
            }
        }
    }
    pub struct MethodInfo {
        data: Vec<MethodInfoEntry>,
    }

    impl MethodInfo {
        pub fn new(data: Vec<MethodInfoEntry>) -> Self {
            Self { data }
        }
    }

    pub struct MethodInfoEntry {
        access_flags: u16,
        name_index: u16,
        descriptor_index: u16,
        attributes_count: u16,
        attribute_info: AttributeInfo,
    }

    impl MethodInfoEntry {
        pub fn new(
            access_flags: u16,
            name_index: u16,
            descriptor_index: u16,
            attributes_count: u16,
            attribute_info: AttributeInfo,
        ) -> Self {
            Self {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attribute_info,
            }
        }
    }
    pub struct AttributeInfo {
        data: Vec<AttributeInfoEntry>,
    }

    impl AttributeInfo {
        pub fn new(data: Vec<AttributeInfoEntry>) -> Self {
            Self { data }
        }
    }

    pub struct AttributeInfoEntry {
        attribute_name_index: u16,
        attribute_length: u32,
        data: Vec<u8>,
    }

    impl AttributeInfoEntry {
        pub fn new(attribute_name_index: u16, attribute_length: u32, data: Vec<u8>) -> Self {
            Self {
                attribute_name_index,
                attribute_length,
                data,
            }
        }
    }
    pub struct InterfaceInfo {
        data: Vec<u16>,
    }

    impl InterfaceInfo {
        pub fn new(data: Vec<u16>) -> Self {
            Self { data }
        }
    }
}

//TODO: reference types
