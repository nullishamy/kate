use parking_lot::lock_api::RwLock;

use std::sync::Arc;
use tracing::debug;

use crate::runtime::heap::object::JVMObject;
use crate::structs::bitflag::{ClassFileAccessFlags, MethodAccessFlags};
use crate::structs::descriptor::MethodDescriptor;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::classfile::MetaData;
use crate::structs::loaded::constant_pool::{ClassData, ConstantPool, Utf8Data};
use crate::structs::loaded::constructors::Constructors;
use crate::structs::loaded::default_attributes::{AttributeEntry, CodeData};
use crate::structs::loaded::field::Fields;
use crate::structs::loaded::interface::Interfaces;
use crate::structs::loaded::method::{MethodEntry, Methods};
use crate::structs::types::{PrimitiveWithValue, RefOrPrim, ReferenceType};
use crate::{LoadedClassFile, MethodAccessFlag};

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
        has_clinit_called: Default::default(),
    };

    let sysout = JVMObject {
        class: Arc::new(sysout),
    };

    class.fields.write().statics.insert(
        "out".to_string(),
        RefOrPrim::Reference(ReferenceType::Class(Arc::new(sysout))),
    );

    let m = *class
        .methods
        .read()
        .entries
        .iter()
        .enumerate()
        .filter(|(i, p)| p.name.str == "getSecurityManager")
        .map(|(i, p)| i)
        .collect::<Vec<usize>>()
        .first()
        .unwrap();

    class.methods.write().entries.remove(m);

    class.methods.write().entries.push(Arc::new(MethodEntry {
        access_flags: MethodAccessFlags::from_bits(
            (MethodAccessFlag::PUBLIC | MethodAccessFlag::STATIC).bits(),
        )
        .unwrap(),
        name: Arc::new(Utf8Data {
            str: "getSecurityManager".to_string(),
        }),
        descriptor: MethodDescriptor::parse(&"()Ljava/lang/SecurityManager;".to_string()).unwrap(),
        attributes: Attributes {
            entries: vec![AttributeEntry::Code(CodeData {
                name: Arc::new(Utf8Data {
                    str: "Code".to_string(),
                }),
                max_stack: 0,
                max_locals: 0,
                // return null, this will bypass the checks
                //FIXME hack
                code: vec![1, 176],
                exception_handlers: vec![],
                attributes: vec![],
            })],
        },
    }));

    debug!("finished writing to java/lang/System");
}

pub fn visit_shutdown(class: Arc<LoadedClassFile>) {
    let m = *class
        .methods
        .read()
        .entries
        .iter()
        .enumerate()
        .filter(|(i, p)| p.name.str == "<clinit>")
        .map(|(i, p)| i)
        .collect::<Vec<usize>>()
        .first()
        .unwrap();

    class.methods.write().entries.remove(m);

    class.methods.write().entries.push(Arc::new(MethodEntry {
        access_flags: MethodAccessFlags::from_bits(
            (MethodAccessFlag::PUBLIC | MethodAccessFlag::STATIC).bits(),
        )
        .unwrap(),
        name: Arc::new(Utf8Data {
            str: "<clinit>".to_string(),
        }),
        descriptor: MethodDescriptor::parse(&"()V".to_string()).unwrap(),
        attributes: Attributes {
            entries: vec![AttributeEntry::Code(CodeData {
                name: Arc::new(Utf8Data {
                    str: "Code".to_string(),
                }),
                max_stack: 0,
                max_locals: 0,
                // return null, this will bypass the checks
                //FIXME hack
                code: vec![177],
                exception_handlers: vec![],
                attributes: vec![],
            })],
        },
    }));

    let m = *class
        .methods
        .read()
        .entries
        .iter()
        .enumerate()
        .filter(|(i, p)| p.name.str == "exit")
        .map(|(i, p)| i)
        .collect::<Vec<usize>>()
        .first()
        .unwrap();

    class.methods.write().entries.remove(m);

    class.methods.write().entries.push(Arc::new(MethodEntry {
        access_flags: MethodAccessFlags::from_bits(
            (MethodAccessFlag::PUBLIC | MethodAccessFlag::STATIC | MethodAccessFlag::NATIVE).bits(),
        )
        .unwrap(),
        name: Arc::new(Utf8Data {
            str: "exit".to_string(),
        }),
        descriptor: MethodDescriptor::parse("(I)V").unwrap(),
        attributes: Attributes { entries: vec![] },
    }));

    debug!("finished writing to java/lang/Shutdown")
}
