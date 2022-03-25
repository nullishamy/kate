use crate::structs::raw::attribute::AttributeEntry;

pub struct FieldEntry {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attribute_info: Vec<AttributeEntry>,
}
