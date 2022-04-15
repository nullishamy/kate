use crate::structs::bitflag::MethodAccessFlags;
use crate::structs::descriptor::MethodDescriptor;
use crate::structs::loaded::attribute::Attributes;
use crate::structs::loaded::constant_pool::Utf8Data;
use std::sync::Arc;

#[derive(Clone)]
pub struct MethodEntry {
    pub access_flags: MethodAccessFlags,
    pub name: Arc<Utf8Data>,
    pub descriptor: MethodDescriptor,
    pub attributes: Attributes,
}

#[derive(Clone)]
pub struct Methods {
    pub entries: Vec<Arc<MethodEntry>>,
}

impl Methods {
    pub fn find<F>(&self, predicate: F) -> Option<Arc<MethodEntry>>
    where
        F: Fn(&&Arc<MethodEntry>) -> bool,
    {
        Some(Arc::clone(self.entries.iter().find(predicate)?))
    }
}
