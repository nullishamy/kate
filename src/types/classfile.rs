use crate::ByteCode;
use anyhow::{anyhow, Result};
use std::borrow::BorrowMut;

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

    pub fn new(tag: u8, byte_code: &ByteCode) -> Result<Self> {
        match tag {
            1 => Ok(ConstantPoolTag::Utf8),
            3 => Ok(ConstantPoolTag::Integer),
            4 => Ok(ConstantPoolTag::Float),
            5 => Ok(ConstantPoolTag::Long),
            6 => Ok(ConstantPoolTag::Double),
            7 => Ok(ConstantPoolTag::Class),
            8 => Ok(ConstantPoolTag::String),
            9 => Ok(ConstantPoolTag::FieldRef),
            10 => Ok(ConstantPoolTag::MethodRef),
            11 => Ok(ConstantPoolTag::InterfaceMethodRef),
            12 => Ok(ConstantPoolTag::NameAndType),
            15 => Ok(ConstantPoolTag::MethodHandle),
            16 => Ok(ConstantPoolTag::MethodType),
            17 => Ok(ConstantPoolTag::Dynamic),
            18 => Ok(ConstantPoolTag::InvokeDynamic),
            19 => Ok(ConstantPoolTag::Module),
            20 => Ok(ConstantPoolTag::Package),
            _ => Err(anyhow!("unknown constant pool tag {}", tag)),
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
