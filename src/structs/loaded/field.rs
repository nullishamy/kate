use std::collections::HashMap;
use std::sync::Arc;

use crate::structs::bitflag::FieldAccessFlags;
use crate::structs::descriptor::FieldDescriptor;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::constant_pool::Utf8Data;
use crate::structs::types::RefOrPrim;

#[derive(Clone, Debug)]
pub struct FieldEntry {
    pub access_flags: FieldAccessFlags,
    pub name: Arc<Utf8Data>,
    pub descriptor: FieldDescriptor,
    pub attributes: Attributes,
}

#[derive(Debug)]
pub struct Fields {
    pub entries: Vec<FieldEntry>,
    pub statics: HashMap<String, RefOrPrim>,
}
