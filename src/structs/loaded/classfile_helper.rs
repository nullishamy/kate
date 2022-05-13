#![allow(unreachable_code)]

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tracing::{debug, warn};

use crate::structs::bitflag::{FieldAccessFlags, MethodAccessFlags};
use crate::structs::descriptor::{FieldDescriptor, MethodDescriptor};
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::constant_pool::Data as LoadedPoolData;
use crate::structs::loaded::constant_pool::PoolEntry as LoadedPoolEntry;
use crate::structs::loaded::constant_pool::{
    ClassData, ConstantPool, DoubleData, FieldRefData, FloatData, IntegerData,
    InterfaceMethodRefData, LongData, MethodHandleData, MethodHandleReference, MethodRefData,
    MethodTypeData, NameAndTypeData, StringData, Utf8Data,
};
use crate::structs::loaded::default_attributes::{AttributeEntry, CodeData};
use crate::structs::loaded::field::{FieldEntry as LoadedFieldEntry, Fields};
use crate::structs::loaded::interface::Interfaces;
use crate::structs::loaded::method::MethodEntry as LoadedMethodEntry;
use crate::structs::loaded::method::Methods;
use crate::structs::raw::attribute::AttributeEntry as RawAttributeEntry;
use crate::structs::raw::constant_pool::Data as RawPoolData;
use crate::structs::raw::constant_pool::{PoolEntry as RawPoolEntry, Tag};
use crate::structs::raw::field::FieldEntry as RawFieldEntry;
use crate::structs::raw::method::MethodEntry as RawMethodEntry;

struct ConstantPoolBuilder {
    pub entries: HashMap<usize, LoadedPoolEntry>,
    pub raw: Vec<Option<RawPoolEntry>>,
    pub major: u16,
}

impl ConstantPoolBuilder {
    fn make_entry(&self, tag: Tag, data: LoadedPoolData) -> LoadedPoolEntry {
        LoadedPoolEntry { tag, data }
    }

    //TODO: reduce this duplication

    // attempt to resolve a string, will transform where possible
    fn string(&self, idx: u16) -> Result<Arc<Utf8Data>> {
        return if let Some(data) = self.entries.get(&(idx as usize)) {
            if let LoadedPoolData::Utf8(data) = &data.data {
                Ok(Arc::clone(data))
            } else {
                Err(anyhow!("pool entry {} was not utf8", idx))
            }
        } else {
            let raw = self.raw.get(idx as usize);

            if raw.is_none() {
                return Err(anyhow!("pool entry {} did not exist", idx));
            }

            let raw = raw.unwrap();

            if raw.is_none() {
                return Err(anyhow!("pool entry {} did not exist", idx));
            }

            let raw = raw.as_ref().unwrap();

            return if let RawPoolData::Utf8(data) = &raw.data {
                Ok(Arc::new(Utf8Data {
                    str: String::from_utf8(data.bytes.clone())?,
                }))
            } else {
                Err(anyhow!("pool entry {} was not utf8", idx))
            };
        };
    }

    fn class(&self, idx: u16) -> Result<Arc<ClassData>> {
        return if let Some(data) = self.entries.get(&(idx as usize)) {
            if let LoadedPoolData::Class(data) = &data.data {
                Ok(Arc::clone(data))
            } else {
                Err(anyhow!("raw pool entry {} was not utf8", idx))
            }
        } else {
            let raw = self.raw.get(idx as usize);

            if raw.is_none() {
                return Err(anyhow!("pool entry {} did not exist", idx));
            }

            let raw = raw.unwrap();

            if raw.is_none() {
                return Err(anyhow!("pool entry {} did not exist", idx));
            }

            let raw = raw.as_ref().unwrap();

            return if let RawPoolData::Class(data) = &raw.data {
                Ok(Arc::new(ClassData {
                    name: self.string(data.name_index)?,
                }))
            } else {
                Err(anyhow!("raw pool entry {} was not class", idx))
            };
        };
    }

    fn name_and_type(&self, idx: u16) -> Result<Arc<NameAndTypeData>> {
        return if let Some(data) = self.entries.get(&(idx as usize)) {
            if let LoadedPoolData::NameAndType(data) = &data.data {
                Ok(Arc::clone(data))
            } else {
                Err(anyhow!("pool entry {} was not utf8", idx))
            }
        } else {
            let raw = self.raw.get(idx as usize);

            if raw.is_none() {
                return Err(anyhow!("pool entry {} did not exist", idx));
            }

            let raw = raw.unwrap();

            if raw.is_none() {
                return Err(anyhow!("pool entry {} did not exist", idx));
            }

            let raw = raw.as_ref().unwrap();

            return if let RawPoolData::NameAndType(data) = &raw.data {
                Ok(Arc::new(NameAndTypeData {
                    name: self.string(data.name_index)?,
                    descriptor: self.string(data.descriptor_index)?,
                }))
            } else {
                Err(anyhow!("raw pool entry {} was not nameandtype", idx))
            };
        };
    }

    fn method_handle_reference(
        &self,
        kind: u8,
        _reference: u16,
        _major: u16,
    ) -> Result<MethodHandleReference> {
        match kind {
            1 | 2 | 3 | 4 => Ok(todo!()),
            5 | 8 => Ok(todo!()),
            6 | 7 => Ok(todo!()),
            9 => Ok(todo!()),
            _ => Err(anyhow!("method handle kind {} out of range", kind)),
        }
    }

    // attempt to transform 1 raw value into a loaded entry
    fn load_one(&self, entry: &RawPoolEntry, major: u16) -> Result<LoadedPoolEntry> {
        match &entry.data {
            RawPoolData::Utf8(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::Utf8(Arc::new(Utf8Data {
                    str: String::from_utf8(data.bytes.clone())?,
                })),
            )),
            RawPoolData::Integer(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::Integer(IntegerData { bytes: data.bytes }),
            )),
            RawPoolData::Float(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::Float(FloatData { bytes: data.bytes }),
            )),
            RawPoolData::Long(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::Long(LongData {
                    low_bytes: data.low_bytes,
                    high_bytes: data.high_bytes,
                }),
            )),
            RawPoolData::Double(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::Double(DoubleData {
                    low_bytes: data.low_bytes,
                    high_bytes: data.high_bytes,
                }),
            )),
            RawPoolData::Class(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::Class(Arc::new(ClassData {
                    name: self.string(data.name_index)?,
                })),
            )),
            RawPoolData::String(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::String(StringData {
                    utf8: self.string(data.utf8_index)?,
                }),
            )),
            RawPoolData::FieldRef(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::FieldRef(Arc::new(FieldRefData {
                    class: self.class(data.class_index)?,
                    name_and_type: self.name_and_type(data.name_and_type_index)?,
                })),
            )),
            RawPoolData::MethodRef(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::MethodRef(MethodRefData {
                    class: self.class(data.class_index)?,
                    name_and_type: self.name_and_type(data.name_and_type_index)?,
                }),
            )),
            RawPoolData::InterfaceMethodRef(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::InterfaceMethodRef(InterfaceMethodRefData {
                    class: self.class(data.class_index)?,
                    name_and_type: self.name_and_type(data.name_and_type_index)?,
                }),
            )),
            RawPoolData::NameAndType(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::NameAndType(Arc::new(NameAndTypeData {
                    name: self.string(data.name_index)?,
                    descriptor: self.string(data.descriptor_index)?,
                })),
            )),
            RawPoolData::MethodHandle(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::MethodHandle(MethodHandleData {
                    reference: self.method_handle_reference(
                        data.reference_kind,
                        data.reference_index,
                        major,
                    )?,
                }),
            )),
            RawPoolData::MethodType(data) => Ok(self.make_entry(
                entry.tag,
                LoadedPoolData::MethodType(MethodTypeData {
                    descriptor: MethodDescriptor::parse(&self.string(data.descriptor_index)?.str)?,
                }),
            )),
            RawPoolData::Dynamic(_) => todo!(), // cannot implement these 4 as they do not have relevant raw types
            RawPoolData::InvokeDynamic(_) => todo!(),
            RawPoolData::Module(_) => todo!(),
            RawPoolData::Package(_) => todo!(),
        }
    }

    pub fn build(&self) -> Result<ConstantPool> {
        let mut pool = ConstantPool {
            entries: HashMap::with_capacity(self.entries.len()),
        };

        let mut idx = 1;
        for entry in &self.raw {
            if entry.is_none() {
                continue;
            }

            let loaded = self.load_one(entry.as_ref().unwrap(), self.major)?;

            // special casing for longs and doubles
            let to_inc = match &loaded.tag {
                Tag::Long => 2,
                Tag::Double => 2,
                _ => 1,
            };

            pool.entries.insert(idx, loaded);

            idx += to_inc;
        }
        Ok(pool)
    }
}

pub fn create_constant_pool(
    raw_constants: Vec<Option<RawPoolEntry>>,
    major: u16,
) -> Result<ConstantPool> {
    let builder = ConstantPoolBuilder {
        entries: HashMap::with_capacity(raw_constants.len()),
        raw: raw_constants,
        major,
    };

    builder.build()
}

pub fn create_interfaces(raw: Vec<u16>, const_pool: &ConstantPool) -> Result<Interfaces> {
    // each entry in 'raw' should be a ClassData entry in the const pool
    // with type interface, relating to a superinterface of the classfile

    let mut out = Interfaces { entries: vec![] };

    for idx in raw {
        let entry = const_pool.class(idx as usize)?;

        //TODO: attach a classloader here and load the entries class so that we can validate it is an interface
        out.entries.push(Arc::clone(&entry))
    }

    Ok(out)
}

pub fn create_fields(raw: Vec<RawFieldEntry>, const_pool: &ConstantPool) -> Result<Fields> {
    let mut out = Fields {
        entries: vec![],
        statics: HashMap::new(),
    };

    debug!("parsing fields");

    for entry in raw {
        let name = const_pool.utf8(entry.name_index as usize)?;
        let name = Arc::clone(&name);

        debug!("field has name {}", name.str);

        let access_flags = FieldAccessFlags::from_bits(entry.access_flags)?;

        debug!("field has access flags {:?}", access_flags.flags);

        let descriptor =
            FieldDescriptor::parse(&const_pool.utf8(entry.descriptor_index as usize)?.str)?;
        let attributes = create_attributes(entry.attribute_info, const_pool)?;

        out.entries.push(LoadedFieldEntry {
            access_flags,
            name,
            descriptor,
            attributes,
        })
    }
    Ok(out)
}

pub fn create_methods(raw: Vec<RawMethodEntry>, const_pool: &ConstantPool) -> Result<Methods> {
    let mut out = Methods { entries: vec![] };

    debug!("parsing methods");
    for entry in raw {
        let name = const_pool.utf8(entry.name_index as usize)?;
        let name = Arc::clone(&name);

        debug!("method has name {}", name.str);

        let access_flags = MethodAccessFlags::from_bits(entry.access_flags)?;
        debug!("method has access flags {:?}", access_flags.flags);

        let descriptor =
            MethodDescriptor::parse(&const_pool.utf8(entry.descriptor_index as usize)?.str)?;

        debug!("method has descriptor {:?}", descriptor);
        let attributes = create_attributes(entry.attribute_info, const_pool)?;
        out.entries.push(Arc::new(LoadedMethodEntry {
            access_flags,
            name,
            descriptor,
            attributes,
        }));
    }
    Ok(out)
}

pub fn create_attributes(
    raw: Vec<RawAttributeEntry>,
    const_pool: &ConstantPool,
) -> Result<Attributes> {
    let mut out = Attributes { entries: vec![] };

    for entry in raw {
        let name = const_pool.utf8(entry.attribute_name_index as usize)?;
        let name = Arc::clone(&name);

        let data = match name.str.as_str() {
            "Code" => AttributeEntry::Code(CodeData::from_bytes(
                name,
                entry.attribute_data,
                const_pool,
            )?),
            _ => {
                warn!("unrecognised attribute '{}'", name.str);

                // ignore attributes we dont recognise
                continue;
            }
        };

        out.entries.push(data)
    }
    Ok(out)
}
