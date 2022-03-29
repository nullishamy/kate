use crate::structs::bitflag::ClassFileAccessFlags;
use anyhow::{anyhow, Result};
use std::rc::Rc;
use tracing::{debug, info};

use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::classfile_helper::{
    create_attributes, create_constant_pool, create_fields, create_interfaces, create_methods,
};
use crate::structs::loaded::constant_pool::{ClassData, ConstantPool};
use crate::structs::loaded::field::Fields;
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
        info!("loading class...");

        let meta = MetaData {
            minor_version: raw.minor_version,
            major_version: raw.major_version,
        };

        info!(
            "class was compiled with version: {major}.{minor}",
            major = meta.major_version,
            minor = meta.minor_version
        );

        let access_flags = ClassFileAccessFlags::from_bits(raw.access_flags)?;

        info!("class has access flags {:?}", access_flags.flags);

        let const_pool = create_constant_pool(raw.const_pool_info, meta.major_version)?;

        info!("constant pool has {} entries", const_pool.entries.len());

        let this_class = Rc::clone(&const_pool.class(raw.this_class as usize)?);

        info!("class has compiled name {}", this_class.name.as_str);

        let mut super_class: Option<Rc<ClassData>> = None;

        if const_pool.has(raw.super_class as usize) {
            let entry = const_pool.class(raw.super_class as usize)?;
            super_class = Some(Rc::clone(&entry));

            info!("class has a superclass, {}", entry.name.as_str);
        } else {
            info!("class has no superclass")
        }

        let interfaces = create_interfaces(raw.interface_info, &const_pool)?;

        if !interfaces.entries.is_empty() {
            info!("class as {} superinterfaces", interfaces.entries.len());

            for interface in &interfaces.entries {
                info!("\t{}", interface.name.as_str)
            }
        } else {
            info!("class has no superinterfaces")
        }

        let fields = create_fields(raw.field_info, &const_pool)?;
        info!("class has {} fields", fields.entries.len());

        let methods = create_methods(raw.method_info, &const_pool)?;
        info!("class has {} methods", methods.entries.len());

        let attributes = create_attributes(raw.attribute_info, &const_pool)?;
        info!("class has {} attributes", attributes.entries.len());

        info!("parsing finished for class {}", this_class.name.as_str);

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
