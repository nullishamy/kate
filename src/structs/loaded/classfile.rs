use crate::structs::bitflag::ClassFileAccessFlags;
use anyhow::{anyhow, Result};
use std::rc::Rc;

use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::classfile_helper::{
    create_attributes, create_constant_pool, create_fields, create_interfaces, create_methods,
};
use crate::structs::loaded::constant_pool::{ClassData, ConstantPool};
use crate::structs::loaded::field::{FieldEntry as LoadedFieldEntry, Fields};
use crate::structs::loaded::interface::Interfaces;
use crate::structs::loaded::method::Methods;
use crate::structs::loaded::package::Package;
use crate::structs::raw::classfile::RawClassFile;

#[derive(Copy, Clone)]
pub struct MetaData {
    minor_version: u16,
    major_version: u16,
}

#[derive(Clone)]
pub struct LoadedClassFile {
    pub const_pool: ConstantPool,
    pub meta: MetaData,

    pub access_flags: ClassFileAccessFlags,
    pub this_class: Rc<ClassData>,
    pub super_class: Option<Rc<ClassData>>,

    pub interfaces: Interfaces,
    pub fields: Fields,
    pub methods: Methods,
    pub attributes: Attributes,
    pub package: Option<Package>,
}

impl LoadedClassFile {
    pub fn from_raw(raw: RawClassFile) -> Result<Self> {
        let meta = MetaData {
            minor_version: raw.minor_version,
            major_version: raw.major_version,
        };

        let access_flags = ClassFileAccessFlags::from_bits(raw.access_flags)?;
        let const_pool = create_constant_pool(raw.const_pool_info, meta.major_version)?;
        let this_class = Rc::clone(&const_pool.class(raw.this_class as usize)?);
        let mut super_class: Option<Rc<ClassData>> = None;

        if const_pool.has(raw.super_class as usize) {
            let entry = const_pool.class(raw.super_class as usize)?;
            super_class = Some(Rc::clone(&entry));
        }

        let interfaces = create_interfaces(raw.interface_info, &const_pool)?;
        let fields = create_fields(raw.field_info, &const_pool)?;
        let methods = create_methods(raw.method_info, &const_pool)?;
        let attributes = create_attributes(raw.attribute_info, &const_pool)?;

        Ok(Self {
            const_pool,
            meta,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
            package: None,
        })
    }
}
