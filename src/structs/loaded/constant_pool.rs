use anyhow::{anyhow, Result};

use crate::structs::descriptor::{MethodDescriptor};
use crate::structs::raw::constant_pool::Tag;
use enum_as_inner::EnumAsInner;
use std::collections::HashMap;

use std::sync::Arc;

#[derive(Clone)]
pub struct ConstantPool {
    pub entries: HashMap<usize, PoolEntry>,
}

impl ConstantPool {
    pub fn get(&self, idx: usize) -> Result<&PoolEntry> {
        let res = self.entries.get(&idx);

        if res.is_none() {
            return Err(anyhow!("pool entry {} does not exist", idx));
        }

        Ok(res.unwrap())
    }

    pub fn class(&self, idx: usize) -> Result<Arc<ClassData>> {
        let entry = self.get(idx)?.data.as_class();

        if entry.is_none() {
            return Err(anyhow!("constant pool entry {} was not a class", idx));
        }

        Ok(Arc::clone(entry.unwrap()))
    }

    pub fn utf8(&self, idx: usize) -> Result<Arc<Utf8Data>> {
        let entry = self.get(idx)?.data.as_utf8();

        if entry.is_none() {
            return Err(anyhow!("constant pool entry {} was not utf8", idx));
        }

        Ok(Arc::clone(entry.unwrap()))
    }

    pub fn field(&self, idx: usize) -> Result<Arc<FieldRefData>> {
        let entry = self.get(idx)?.data.as_field_ref();

        if entry.is_none() {
            return Err(anyhow!("constant pool entry {} was not a class", idx));
        }

        Ok(Arc::clone(entry.unwrap()))
    }

    pub fn has(&self, idx: usize) -> bool {
        self.entries.contains_key(&idx)
    }
}

#[derive(Clone)]
pub struct Utf8Data {
    pub str: String,
}

impl Into<String> for Utf8Data {
    fn into(self) -> String {
        self.str
    }
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
    pub low_bytes: u32,
    pub high_bytes: u32,
}

#[derive(Copy, Clone)]
pub struct DoubleData {
    pub low_bytes: f32,
    pub high_bytes: f32,
}

#[derive(Clone)]
pub struct ClassData {
    pub name: Arc<Utf8Data>,
}

#[derive(Clone)]
pub struct StringData {
    pub utf8: Arc<Utf8Data>,
}

#[derive(Clone)]
pub struct FieldRefData {
    pub class: Arc<ClassData>,
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct MethodRefData {
    pub class: Arc<ClassData>,
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct InterfaceMethodRefData {
    pub class: Arc<ClassData>,
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct NameAndTypeData {
    pub name: Arc<Utf8Data>,
    pub descriptor: Arc<Utf8Data>, // we cannot specify which descriptor this will be trivially, so we must parse and store it later
}

#[derive(Clone)]
pub enum MethodHandleReference {
    FieldRef(FieldRefData),
    MethodRef(MethodRefData),
    InterfaceMethodRef(InterfaceMethodRefData),
}

#[derive(Clone)]
pub struct MethodHandleData {
    pub reference: MethodHandleReference,
}

#[derive(Clone)]
pub struct MethodTypeData {
    pub descriptor: MethodDescriptor,
}

#[derive(Clone)]
pub struct DynamicData {
    pub bootstrap_method_attr_index: u16, //TODO: resolve this when attribute parsing is implemented
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct InvokeDynamicData {
    pub bootstrap_method_attr_index: u16, //TODO: resolve this when attribute parsing is implemented
    pub name_and_type: Arc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct ModuleData {
    pub name: Arc<Utf8Data>,
}

#[derive(Clone)]
pub struct PackageData {
    pub name: Arc<Utf8Data>,
}

#[derive(Clone, EnumAsInner)]
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

#[derive(Clone)]
pub struct PoolEntry {
    pub tag: Tag,
    pub data: Data,
}
