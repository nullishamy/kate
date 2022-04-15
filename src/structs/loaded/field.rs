use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::heap::object::JVMObject;
use crate::structs::bitflag::FieldAccessFlags;
use crate::structs::descriptor::FieldDescriptor;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::constant_pool::Utf8Data;

#[derive(Clone)]
pub struct FieldEntry {
    pub access_flags: FieldAccessFlags,
    pub name: Arc<Utf8Data>,
    pub descriptor: FieldDescriptor,
    pub attributes: Attributes,
}

pub struct Fields {
    pub entries: Vec<FieldEntry>,
    pub statics: HashMap<String, Arc<JVMObject>>,
}
