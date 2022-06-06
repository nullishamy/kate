use core::result::Result::{Err, Ok};

use anyhow::{anyhow, Result};
use enum_as_inner::EnumAsInner;

#[derive(Copy, Clone, Debug)]
pub enum Tag {
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

impl Tag {
    pub fn from_tag_byte(tag: u8) -> Result<Self> {
        Ok(match tag {
            1 => Tag::Utf8,
            3 => Tag::Integer,
            4 => Tag::Float,
            5 => Tag::Long,
            6 => Tag::Double,
            7 => Tag::Class,
            8 => Tag::String,
            9 => Tag::FieldRef,
            10 => Tag::MethodRef,
            11 => Tag::InterfaceMethodRef,
            12 => Tag::NameAndType,
            15 => Tag::MethodHandle,
            16 => Tag::MethodType,
            17 => Tag::Dynamic,
            18 => Tag::InvokeDynamic,
            19 => Tag::Module,
            20 => Tag::Package,
            _ => return Err(anyhow!("unknown constant pool tag {}", tag)),
        })
    }

    pub fn loadable(&self) -> bool {
        matches!(
            self,
            Tag::Integer
                | Tag::Float
                | Tag::Long
                | Tag::Double
                | Tag::Class
                | Tag::String
                | Tag::MethodHandle
                | Tag::MethodType
                | Tag::Dynamic
        )
    }
}

#[derive(Clone, Debug)]
pub struct Utf8Data {
    pub length: u16,
    pub bytes: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
pub struct IntegerData {
    pub bytes: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct FloatData {
    pub bytes: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct LongData {
    // TODO: this has to take up 2 entries (?)
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct DoubleData {
    // TODO: this has to take up 2 entries (?)
    pub low_bytes: f32,
    pub high_bytes: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct ClassData {
    pub name_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct StringData {
    pub utf8_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct FieldRefData {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct MethodRefData {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct InterfaceMethodRefData {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct NameAndTypeData {
    pub name_index: u16,
    pub descriptor_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct MethodHandleData {
    pub reference_kind: u8,
    pub reference_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct MethodTypeData {
    pub descriptor_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct DynamicData {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct InvokeDynamicData {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct ModuleData {
    pub name_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct PackageData {
    pub name_index: u16,
}

#[derive(Clone, EnumAsInner, Debug)]
pub enum Data {
    Utf8(Utf8Data),
    Integer(IntegerData),
    Float(FloatData),
    Long(LongData),
    Double(DoubleData),
    Class(ClassData),
    String(StringData),
    FieldRef(FieldRefData),
    MethodRef(MethodRefData),
    InterfaceMethodRef(InterfaceMethodRefData),
    NameAndType(NameAndTypeData),
    MethodHandle(MethodHandleData),
    MethodType(MethodTypeData),
    Dynamic(DynamicData),
    InvokeDynamic(InvokeDynamicData),
    Module(ModuleData),
    Package(PackageData),
}

#[derive(Clone, Debug)]
pub struct PoolEntry {
    pub tag: Tag,
    pub data: Data,
}
