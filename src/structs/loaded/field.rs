use crate::structs::bitflag::FieldAccessFlags;
use crate::structs::descriptor::FieldDescriptor;
use crate::structs::loaded::attribute::{AttributeEntry, Attributes};

#[derive(Clone)]
pub struct FieldEntry {
    pub access_flags: FieldAccessFlags,
    pub name: String,
    pub descriptor: FieldDescriptor,
    pub attributes: Attributes,
}

#[derive(Clone)]
pub struct Fields {
    pub entries: Vec<FieldEntry>,
}
