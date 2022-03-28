use crate::structs::loaded::constant_pool::Utf8Data;
use crate::structs::loaded::default_attributes::{AttributeEntry, CustomData};
use anyhow::{anyhow, Result};
use std::rc::Rc;

#[derive(Clone)]
pub struct Attributes {
    pub entries: Vec<AttributeEntry>,
}

impl Attributes {
    pub fn get(&self, key: &str) -> Option<&AttributeEntry> {
        // deref one layer, the vec lookup
        self.entries
            .iter()
            .filter(|a| a.name().as_str == key)
            .collect::<Vec<&AttributeEntry>>()
            .first()
            .map(|r| *r)
    }
}
