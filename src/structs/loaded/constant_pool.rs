use anyhow::{anyhow, Result};

use crate::structs::descriptor::{Descriptor, MethodDescriptor};
use crate::structs::raw::constant_pool::Tag;
use std::collections::HashMap;
use std::rc::Rc;

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

    pub fn class(&self, idx: usize) -> Result<&Rc<ClassData>> {
        let entry = self.get(idx)?;

        match &entry.data {
            Data::Class(data) => Ok(data),
            _ => Err(anyhow!("constant pool entry {} was not a class", idx)),
        }
    }

    pub fn utf8(&self, idx: usize) -> Result<&Rc<Utf8Data>> {
        let entry = self.get(idx)?;

        match &entry.data {
            Data::Utf8(data) => Ok(data),
            _ => Err(anyhow!("constant pool entry {} was not utf8", idx)),
        }
    }

    pub fn has(&self, idx: usize) -> bool {
        self.entries.contains_key(&idx)
    }
}

#[derive(Clone)]
pub struct Utf8Data {
    pub as_str: String,
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

#[derive(Clone)]
pub struct ClassData {
    pub name: Rc<Utf8Data>,
}

#[derive(Clone)]
pub struct StringData {
    pub utf8: Rc<Utf8Data>,
}

#[derive(Clone)]
pub struct FieldRefData {
    pub class: Rc<ClassData>,
    pub name_and_type: Rc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct MethodRefData {
    pub class: Rc<ClassData>,
    pub name_and_type: Rc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct InterfaceMethodRefData {
    pub class: Rc<ClassData>,
    pub name_and_type: Rc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct NameAndTypeData {
    pub name: Rc<Utf8Data>,
    pub descriptor: Rc<Utf8Data>, // we cannot specify which descriptor this will be trivially, so we must parse and store it later
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
    pub name_and_type: Rc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct InvokeDynamicData {
    pub bootstrap_method_attr_index: u16, //TODO: resolve this when attribute parsing is implemented
    pub name_and_type: Rc<NameAndTypeData>,
}

#[derive(Clone)]
pub struct ModuleData {
    pub name: Rc<Utf8Data>,
}

#[derive(Clone)]
pub struct PackageData {
    pub name: Rc<Utf8Data>,
}

#[derive(Clone)]
pub enum Data {
    Utf8(Rc<Utf8Data>),
    Integer(IntegerData),
    Float(FloatData),
    Long(LongData),
    Double(DoubleData),
    Class(Rc<ClassData>),
    String(StringData),
    FieldRef(FieldRefData),
    MethodRef(MethodRefData),
    InterfaceMethodRef(InterfaceMethodRefData),
    NameAndType(Rc<NameAndTypeData>),
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
