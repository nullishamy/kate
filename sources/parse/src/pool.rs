use std::rc::Rc;

use anyhow::Result;
use enum_as_inner::EnumAsInner;
use parking_lot::RwLock;

use crate::classfile::Addressed;
use crate::classfile::Resolvable;

#[derive(Debug, Clone)]
pub struct ConstantPool {
    pub entries: Rc<RwLock<Vec<ConstantEntry>>>,
}

impl Default for ConstantPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstantPool {
    pub fn new() -> Self {
        Self {
            entries: Rc::new(RwLock::new(vec![])),
        }
    }

    pub fn insert(&mut self, entry: ConstantEntry) {
        let mut pool = self.entries.write();
        pool.push(entry)
    }

    pub fn get(&self, index: u16) -> Option<ConstantEntry> {
        let pool = self.entries.read();
        pool.get(index as usize).cloned()
    }

    pub fn address<T>(&self, for_index: u16) -> Addressed<T> {
        Addressed::from(for_index, Rc::clone(&self.entries))
    }

    pub(crate) fn perform_format_checking(&self) -> Result<()> {
        let entries = self.entries.read();
        for item in entries.iter() {
            match item {
                ConstantEntry::Class(data) => {
                    data.name.try_resolve()?;
                }
                ConstantEntry::Field(data) => {
                    data.class.try_resolve()?;
                    data.name_and_type.try_resolve()?;
                }
                ConstantEntry::Method(data) => {
                    data.class.try_resolve()?;
                    data.name_and_type.try_resolve()?;
                }
                ConstantEntry::InterfaceMethod(data) => {
                    data.class.try_resolve()?;
                    data.name_and_type.try_resolve()?;
                }
                ConstantEntry::String(data) => {
                    data.string.try_resolve()?;
                }
                ConstantEntry::Integer(_) => {}
                ConstantEntry::Float(_) => {}
                ConstantEntry::Long(_) => {}
                ConstantEntry::Double(_) => {}
                ConstantEntry::NameAndType(data) => {
                    data.name.try_resolve()?;
                    data.descriptor.try_resolve()?;
                }
                ConstantEntry::Utf8(_) => {}
                ConstantEntry::MethodHandle(_) => {
                    // TODO: Validate this once we figure out other data structures
                }
                ConstantEntry::MethodType(data) => {
                    data.descriptor.try_resolve()?;
                }
                ConstantEntry::Dynamic(_) => todo!(),
                ConstantEntry::InvokeDynamic(data) => {
                    // TODO: Validate data.method_index once we figure out that implementation
                    data.name_and_type.try_resolve()?;
                }
                ConstantEntry::Package(_) => todo!(),
                ConstantEntry::Reserved => {}
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConstantTag {
    Class,
    Field,
    Method,
    InterfaceMethod,
    String,
    Integer,
    Float,
    Long,
    Double,
    NameAndType,
    Utf8,
    MethodHandle,
    MethodType,
    Dynamic,
    InvokeDynamic,
    Module,
    Package,
}

impl ConstantTag {
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            1 => ConstantTag::Utf8,
            3 => ConstantTag::Integer,
            4 => ConstantTag::Float,
            5 => ConstantTag::Long,
            6 => ConstantTag::Double,
            7 => ConstantTag::Class,
            8 => ConstantTag::String,
            9 => ConstantTag::Field,
            10 => ConstantTag::Method,
            11 => ConstantTag::InterfaceMethod,
            12 => ConstantTag::NameAndType,
            15 => ConstantTag::MethodHandle,
            16 => ConstantTag::MethodType,
            17 => ConstantTag::Dynamic,
            18 => ConstantTag::InvokeDynamic,
            19 => ConstantTag::Method,
            20 => ConstantTag::Package,
            _ => todo!("{} is an unknown tag", tag),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantClass {
    pub tag: ConstantTag,
    pub name: Addressed<ConstantUtf8>,
}

#[derive(Debug, Clone)]
pub struct ConstantField {
    pub tag: ConstantTag,
    pub class: Addressed<ConstantClass>,
    pub name_and_type: Addressed<ConstantNameAndType>,
}

#[derive(Debug, Clone)]
pub struct ConstantMethod {
    pub tag: ConstantTag,
    pub class: Addressed<ConstantClass>,
    pub name_and_type: Addressed<ConstantNameAndType>,
}

#[derive(Debug, Clone)]
pub struct ConstantInterfaceMethod {
    pub tag: ConstantTag,
    pub class: Addressed<ConstantClass>,
    pub name_and_type: Addressed<ConstantNameAndType>,
}
#[derive(Debug, Clone)]
pub struct ConstantString {
    pub tag: ConstantTag,
    pub string: Addressed<ConstantUtf8>,
}
#[derive(Debug, Clone)]
pub struct ConstantInteger {
    pub tag: ConstantTag,
    pub bytes: u32,
}

#[derive(Debug, Clone)]
pub struct ConstantFloat {
    pub tag: ConstantTag,
    pub bytes: f32,
}

#[derive(Debug, Clone)]
pub struct ConstantLong {
    pub tag: ConstantTag,
    pub bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ConstantDouble {
    pub tag: ConstantTag,
    pub bytes: f64,
}
#[derive(Debug, Clone)]
pub struct ConstantNameAndType {
    pub tag: ConstantTag,
    pub name: Addressed<ConstantUtf8>,
    pub descriptor: Addressed<ConstantUtf8>,
}

#[derive(Debug, Clone)]
pub struct ConstantUtf8 {
    pub tag: ConstantTag,
    pub length: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ConstantMethodHandle {
    pub tag: ConstantTag,
    pub kind: u8,
    pub index: u16,
}
#[derive(Debug, Clone)]
pub struct ConstantMethodType {
    pub tag: ConstantTag,
    pub descriptor: Addressed<ConstantUtf8>,
}

#[derive(Debug, Clone)]
pub struct ConstantDynamic {
    pub tag: ConstantTag,
    pub method_index: u16,
    pub name_and_type: Addressed<ConstantNameAndType>,
}

#[derive(Debug, Clone)]
pub struct ConstantInvokeDynamic {
    pub tag: ConstantTag,
    pub method_index: u16,
    pub name_and_type: Addressed<ConstantNameAndType>,
}

#[derive(Debug, Clone)]
pub struct ConstantPackage {
    pub tag: ConstantTag,
    pub name: Addressed<ConstantUtf8>,
}

// TODO: pretty sure these need to use a utf8 variant
impl ConstantUtf8 {
    pub fn string(self) -> String {
        String::from_utf8(self.bytes).unwrap()
    }

    pub fn try_string(self) -> Result<String> {
        Ok(String::from_utf8(self.bytes)?)
    }
}

impl ConstantString {
    pub fn string(&self) -> String {
        String::from_utf8(self.string.resolve().bytes).unwrap()
    }

    pub fn try_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.string.try_resolve()?.bytes)?)
    }
}

#[derive(EnumAsInner, Clone, Debug)]
pub enum ConstantEntry {
    Class(ConstantClass),
    Field(ConstantField),
    Method(ConstantMethod),
    InterfaceMethod(ConstantInterfaceMethod),
    String(ConstantString),
    Integer(ConstantInteger),
    Float(ConstantFloat),
    Long(ConstantLong),
    Double(ConstantDouble),
    NameAndType(ConstantNameAndType),
    Utf8(ConstantUtf8),
    MethodHandle(ConstantMethodHandle),
    MethodType(ConstantMethodType),
    Dynamic(ConstantDynamic),
    InvokeDynamic(ConstantInvokeDynamic),
    Package(ConstantPackage),
    Reserved,
}

#[cfg(test)]
mod tests {}
