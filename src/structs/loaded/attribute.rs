use crate::structs::loaded::default_attributes::AttributeEntry;

#[derive(Clone, Debug)]
pub struct Attributes {
    pub entries: Vec<AttributeEntry>,
}

impl Attributes {
    pub fn get(&self, key: &str) -> Option<&AttributeEntry> {
        self.entries.iter().find(|a| a.name().str == key)
    }
}
