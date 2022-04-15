
use crate::structs::loaded::default_attributes::{AttributeEntry};



#[derive(Clone)]
pub struct Attributes {
    pub entries: Vec<AttributeEntry>,
}

impl Attributes {
    pub fn get(&self, key: &str) -> Option<&AttributeEntry> {
        self.entries
            .iter()
            .filter(|a| a.name().str == key)
            .collect::<Vec<&AttributeEntry>>()
            .first()
            .copied()
    }
}
