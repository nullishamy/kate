use crate::ByteCode;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct ConstantPoolInfo {
    data: HashMap<u16, ConstantPoolEntry>,
}

impl ConstantPoolInfo {
    pub fn new(data: HashMap<u16, ConstantPoolEntry>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &HashMap<u16, ConstantPoolEntry> {
        &self.data
    }

    pub fn get(&self, idx: u16) -> Result<&ConstantPoolEntry> {
        if let Some(v) = self.data.get(&idx) {
            return Ok(v);
        }
        Err(anyhow!("constant pool key {} does not exist", idx))
    }

    pub fn has(&self, idx: u16) -> bool {
        self.data.contains_key(&idx)
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

    pub fn data(&self) -> &ConstantPoolData {
        &self.data
    }

    pub fn tag(&self) -> &ConstantPoolTag {
        &self.tag
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
        matches!(
            self,
            ConstantPoolTag::Integer
                | ConstantPoolTag::Float
                | ConstantPoolTag::Long
                | ConstantPoolTag::Double
                | ConstantPoolTag::Class
                | ConstantPoolTag::String
                | ConstantPoolTag::MethodHandle
                | ConstantPoolTag::MethodType
                | ConstantPoolTag::Dynamic
        )
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

    pub fn data(&self) -> &Vec<MethodInfoEntry> {
        &self.data
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
    
    pub fn name_index(&self) -> u16 {
        self.name_index
    }

    pub fn attributes(&self) -> &AttributeInfo {
        return &self.attribute_info;
    }
}
pub struct AttributeInfo {
    data: Vec<AttributeInfoEntry>,
}

impl AttributeInfo {
    pub fn new(data: Vec<AttributeInfoEntry>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &Vec<AttributeInfoEntry> {
        &self.data
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

    pub fn name_index(&self) -> u16 {
        self.attribute_name_index
    }

    pub fn len(&self) -> u32 {
        self.attribute_length
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
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
