use crate::structs::bitflag::FieldAccessFlags;
use crate::structs::descriptor::FieldDescriptor;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::constant_pool::Utf8Data;
use crate::structs::loaded::default_attributes::CustomData;
use std::rc::Rc;

#[derive(Clone)]
pub struct FieldEntry {
    pub access_flags: FieldAccessFlags,
    pub name: Rc<Utf8Data>,
    pub descriptor: FieldDescriptor,
    pub attributes: Attributes,
}

#[derive(Clone)]
pub struct Fields {
    pub entries: Vec<FieldEntry>,
}
