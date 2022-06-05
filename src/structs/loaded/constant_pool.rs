use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use enum_as_inner::EnumAsInner;

use crate::structs::descriptor::MethodDescriptor;
use crate::structs::raw::constant_pool::Tag;

#[derive(Clone, Debug)]
pub struct ConstantPool {
    pub entries: HashMap<usize, PoolEntry>,
}

impl ConstantPool {
    pub fn get(&self, idx: usize) -> Result<&PoolEntry> {
        let res = self.entries.get(&idx);
        res.ok_or_else(|| anyhow!("pool entry {} does not exist", idx))
    }

    pub fn class(&self, idx: usize) -> Result<Arc<ClassData>> {
        let entry = self.get(idx)?.data.as_class();
        let entry = entry.ok_or_else(|| anyhow!("constant pool entry {} was not a class", idx))?;

        Ok(Arc::clone(entry))
    }

    pub fn utf8(&self, idx: usize) -> Result<Arc<Utf8Data>> {
        let entry = self.get(idx)?.data.as_utf8();
        let entry = entry.ok_or_else(|| anyhow!("constant pool entry {} was not utf8", idx))?;

        Ok(Arc::clone(entry))
    }

    pub fn field(&self, idx: usize) -> Result<Arc<FieldRefData>> {
        let entry = self.get(idx)?.data.as_field_ref();
        let entry = entry
            .ok_or_else(|| anyhow!("constant pool entry {} was not a field reference", idx))?;

        Ok(Arc::clone(entry))
    }

    pub fn has(&self, idx: usize) -> bool {
        self.entries.contains_key(&idx)
    }
}

#[derive(Clone, Debug)]
pub struct Utf8Data {
    pub str: String,
}

impl From<String> for Utf8Data {
    fn from(str: String) -> Self {
        Self { str }
    }
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
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct DoubleData {
    pub low_bytes: f32,
    pub high_bytes: f32,
}

#[derive(Clone, Debug)]
pub struct ClassData {
    pub name: Arc<Utf8Data>,
}

#[derive(Clone, Debug)]
pub struct StringData {
    pub utf8: Arc<Utf8Data>,
}

#[derive(Clone, Debug)]
pub struct FieldRefData {
    pub class: Arc<ClassData>,
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone, Debug)]
pub struct MethodRefData {
    pub class: Arc<ClassData>,
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone, Debug)]
pub struct InterfaceMethodRefData {
    pub class: Arc<ClassData>,
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone, Debug)]
pub struct NameAndTypeData {
    pub name: Arc<Utf8Data>,
    pub descriptor: Arc<Utf8Data>, // we cannot specify which descriptor this will be trivially, so we must parse and store it later
}

#[derive(Clone, Debug)]
pub enum MethodHandleReference {
    Field(FieldRefData),
    Method(MethodRefData),
    InterfaceMethod(InterfaceMethodRefData),
}

#[derive(Clone, Debug)]
pub struct MethodHandleData {
    pub reference: MethodHandleReference,
}

#[derive(Clone, Debug)]
pub struct MethodTypeData {
    pub descriptor: MethodDescriptor,
}

#[derive(Clone, Debug)]
pub struct DynamicData {
    pub bootstrap_method_attr_index: u16,
    //TODO: resolve this when attribute parsing is implemented
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone, Debug)]
pub struct InvokeDynamicData {
    pub bootstrap_method_attr_index: u16,
    //TODO: resolve this when attribute parsing is implemented
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone, Debug)]
pub struct ModuleData {
    pub name: Arc<Utf8Data>,
}

#[derive(Clone, Debug)]
pub struct PackageData {
    pub name: Arc<Utf8Data>,
}

#[derive(Clone, EnumAsInner, Debug)]
pub enum Data {
    Utf8(Arc<Utf8Data>),
    Integer(IntegerData),
    Float(FloatData),
    Long(LongData),
    Double(DoubleData),
    Class(Arc<ClassData>),
    String(StringData),
    FieldRef(Arc<FieldRefData>),
    MethodRef(MethodRefData),
    InterfaceMethodRef(InterfaceMethodRefData),
    NameAndType(Arc<NameAndTypeData>),
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
