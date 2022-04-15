use parking_lot::lock_api::RwLock;
use std::ops::Deref;
use std::sync::Arc;

use crate::runtime::heap::object::JVMObject;
use crate::structs::bitflag::ClassFileAccessFlags;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::classfile::MetaData;
use crate::structs::loaded::constant_pool::{ClassData, ConstantPool, Utf8Data};
use crate::structs::loaded::constructors::Constructors;
use crate::structs::loaded::field::Fields;
use crate::structs::loaded::interface::Interfaces;
use crate::structs::loaded::method::Methods;
use crate::LoadedClassFile;

pub fn visit_system(class: Arc<LoadedClassFile>) {
    let sysout = LoadedClassFile {
        const_pool: ConstantPool {
            entries: Default::default(),
        },
        meta: MetaData {
            minor_version: 0,
            major_version: 62,
        },
        access_flags: ClassFileAccessFlags::from_bits(000000).unwrap(),
        this_class: Arc::new(ClassData {
            name: Arc::new(Utf8Data {
                str: "sysout".to_string(),
            }),
        }),
        super_class: None,
        interfaces: Interfaces { entries: vec![] },
        fields: RwLock::new(Fields {
            entries: vec![],
            statics: Default::default(),
        }),
        methods: RwLock::new(Methods { entries: vec![] }),
        constructors: Constructors { entries: vec![] },
        attributes: Attributes { entries: vec![] },
        package: None,
    };

    let sysout = JVMObject {
        class: Arc::new(sysout),
    };

    class
        .fields
        .write()
        .statics
        .insert("out".to_string(), Arc::new(sysout));
}
