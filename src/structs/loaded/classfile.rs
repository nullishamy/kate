
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::runtime::bytecode::args::Args;
use crate::runtime::heap::object::JVMObject;
use crate::structs::bitflag::ClassFileAccessFlags;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::classfile_helper::{
    create_attributes, create_constant_pool, create_fields, create_interfaces, create_methods,
};
use crate::structs::loaded::constant_pool::{ClassData, ConstantPool};
use crate::structs::loaded::constructors::Constructors;
use crate::structs::loaded::field::Fields;
use crate::structs::loaded::interface::Interfaces;
use crate::structs::loaded::method::Methods;
use crate::structs::loaded::package::Package;
use crate::structs::raw::classfile::RawClassFile;
use crate::{CallSite, VM};

#[derive(Copy, Clone, Debug)]
pub struct MetaData {
    pub minor_version: u16,
    pub major_version: u16,
}

#[derive(Debug)]
pub struct LoadedClassFile {
    pub const_pool: ConstantPool,
    pub meta: MetaData,

    pub access_flags: ClassFileAccessFlags,
    pub this_class: Arc<ClassData>,
    pub super_class: Option<Arc<ClassData>>,

    pub interfaces: Interfaces,
    pub fields: RwLock<Fields>,
    pub methods: RwLock<Methods>,
    pub constructors: Constructors,
    pub attributes: Attributes,
    pub package: Option<Package>,

    pub has_clinit_called: AtomicBool,
}

impl LoadedClassFile {
    pub fn from_raw(raw: RawClassFile) -> Result<Self> {
        debug!("loading class...");

        let meta = MetaData {
            minor_version: raw.minor_version,
            major_version: raw.major_version,
        };

        debug!(
            "class was compiled with version: {major}.{minor}",
            major = meta.major_version,
            minor = meta.minor_version
        );

        let access_flags = ClassFileAccessFlags::from_bits(raw.access_flags)?;

        debug!("class has access flags {:?}", access_flags.flags);

        let const_pool = create_constant_pool(raw.const_pool_info, meta.major_version)?;

        debug!("constant pool has {} entries", const_pool.entries.len());

        let this_class = Arc::clone(&const_pool.class(raw.this_class as usize)?);

        debug!("class has compiled name {}", this_class.name.str);

        let mut super_class: Option<Arc<ClassData>> = None;

        if const_pool.has(raw.super_class as usize) {
            let entry = const_pool.class(raw.super_class as usize)?;
            super_class = Some(Arc::clone(&entry));

            debug!("class has a superclass, {}", entry.name.str);
        } else {
            debug!("class has no superclass")
        }

        let interfaces = create_interfaces(raw.interface_info, &const_pool)?;

        if !interfaces.entries.is_empty() {
            debug!("class as {} superinterfaces", interfaces.entries.len());

            for interface in &interfaces.entries {
                debug!("\t{}", interface.name.str)
            }
        } else {
            debug!("class has no superinterfaces")
        }

        let fields = RwLock::new(create_fields(raw.field_info, &const_pool)?);
        debug!("class has {} fields", fields.read().entries.len());

        let methods = RwLock::new(create_methods(raw.method_info, &const_pool)?);
        debug!("class has {} methods", methods.read().entries.len());

        let constructors = Constructors::from_methods(&methods.read());
        debug!("class has {} constructors", constructors.entries.len());

        let attributes = create_attributes(raw.attribute_info, &const_pool)?;
        debug!("class has {} attributes", attributes.entries.len());

        info!("loading finished for class {}", this_class.name.str);

        Ok(Self {
            const_pool,
            meta,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            constructors,
            attributes,
            package: None, //TODO: implement packages
            has_clinit_called: AtomicBool::new(false),
        })
    }

    pub fn new_instance(self: Arc<Self>, vm: &VM) -> Result<Arc<JVMObject>> {
        let obj = JVMObject {
            class: Arc::clone(&self),
        };

        vm.heap.write().push(obj)
    }

    pub fn requires_clinit(&self) -> bool {
        !self.has_clinit_called.load(Ordering::Acquire)
    }

    pub fn run_clinit(self: &Arc<Self>, vm: &VM, ctx: CallSite) -> Result<()> {
        if !self.requires_clinit() {
            debug!("clinit already called for {}", self.this_class.name.str);
            return Ok(());
        }

        let clinit = self.methods.read().find(|m| m.name.str == "<clinit>");

        if clinit.is_none() {
            warn!("clinit not found for {}", self.this_class.name.str);
            return Ok(());
        }

        debug!("storing clinit state {}", self.this_class.name.str);
        self.has_clinit_called.store(true, Ordering::Release);

        debug!("calling clinit for {}", self.this_class.name.str);
        let res = vm.interpret(
            CallSite::new(Arc::clone(self), ctx.thread, clinit.unwrap(), None),
            Args { entries: vec![] }, // empty args for clinit. it cannot take args, it is a class initialiser
            false,
        );

        debug!("clinit finished for {}", ctx.class.this_class.name.str);
        res
    }
}
