use anyhow::{anyhow, Result};
use core::result::Result::{Err, Ok};
use enum_as_inner::EnumAsInner;

#[derive(Copy, Clone)]
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
        match tag {
            1 => Ok(Tag::Utf8),
            3 => Ok(Tag::Integer),
            4 => Ok(Tag::Float),
            5 => Ok(Tag::Long),
            6 => Ok(Tag::Double),
            7 => Ok(Tag::Class),
            8 => Ok(Tag::String),
            9 => Ok(Tag::FieldRef),
            10 => Ok(Tag::MethodRef),
            11 => Ok(Tag::InterfaceMethodRef),
            12 => Ok(Tag::NameAndType),
            15 => Ok(Tag::MethodHandle),
            16 => Ok(Tag::MethodType),
            17 => Ok(Tag::Dynamic),
            18 => Ok(Tag::InvokeDynamic),
            19 => Ok(Tag::Module),
            20 => Ok(Tag::Package),
            _ => Err(anyhow!("unknown constant pool tag {}", tag)),
        }
    }

    fn loadable(&self) -> bool {
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

#[derive(Clone)]
pub struct Utf8Data {
    pub length: u16,
    pub bytes: Vec<u8>,
}

#[derive(Copy, Clone)]
pub struct IntegerData {
    pub bytes: u32,
}

#[derive(Copy, Clone)]
pub struct FloatData {
    pub bytes: f32,
}

#[derive(Copy, Clone)]
pub struct LongData {
    //TODO: this has to take up 2 entries (?)
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Copy, Clone)]
pub struct DoubleData {
    //TODO: this has to take up 2 entries (?)
    pub low_bytes: f32,
    pub high_bytes: f32,
}

#[derive(Copy, Clone)]
pub struct ClassData {
    pub name_index: u16,
}

#[derive(Copy, Clone)]
pub struct StringData {
    pub utf8_index: u16,
}

#[derive(Copy, Clone)]
pub struct FieldRefData {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone)]
pub struct MethodRefData {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone)]
pub struct InterfaceMethodRefData {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone)]
pub struct NameAndTypeData {
    pub name_index: u16,
    pub descriptor_index: u16,
}

#[derive(Copy, Clone)]
pub struct MethodHandleData {
    pub reference_kind: u8,
    pub reference_index: u16,
}

#[derive(Copy, Clone)]
pub struct MethodTypeData {
    pub descriptor_index: u16,
}

#[derive(Copy, Clone)]
pub struct DynamicData {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone)]
pub struct InvokeDynamicData {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Copy, Clone)]
pub struct ModuleData {
    pub name_index: u16,
}

#[derive(Copy, Clone)]
pub struct PackageData {
    pub name_index: u16,
}

#[derive(Clone, EnumAsInner)]
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

#[derive(Clone)]
pub struct PoolEntry {
    pub tag: Tag,
    pub data: Data,
}
