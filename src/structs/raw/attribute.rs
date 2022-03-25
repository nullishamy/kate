#[derive(Clone)]
pub struct AttributeEntry {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub attribute_data: Vec<u8>,
}
