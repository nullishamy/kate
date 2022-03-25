use crate::structs::bitflag::MethodAccessFlags;
use crate::structs::descriptor::MethodDescriptor;
use crate::structs::loaded::attribute::Attributes;

#[derive(Clone)]
pub struct MethodEntry {
    pub access_flags: MethodAccessFlags,
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub attributes: Attributes,
}

#[derive(Clone)]
pub struct Methods {
    pub entries: Vec<MethodEntry>,
}
