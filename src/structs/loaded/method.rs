use crate::structs::bitflag::MethodAccessFlags;
use crate::structs::descriptor::MethodDescriptor;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::constant_pool::Utf8Data;
use std::rc::Rc;

#[derive(Clone)]
pub struct MethodEntry {
    pub access_flags: MethodAccessFlags,
    pub name: Rc<Utf8Data>,
    pub descriptor: MethodDescriptor,
    pub attributes: Attributes,
}

#[derive(Clone)]
pub struct Methods {
    pub entries: Vec<MethodEntry>,
}

impl Methods {
    pub fn find<F>(&self, predicate: F) -> Option<&MethodEntry>
    where
        F: Fn(&&MethodEntry) -> bool,
    {
        self.entries.iter().find(predicate)
    }
}
