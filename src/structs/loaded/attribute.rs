#[derive(Clone)]
pub struct AttributeEntry {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct Attributes {
    pub entries: Vec<AttributeEntry>,
}
