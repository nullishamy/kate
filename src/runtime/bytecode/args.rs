use crate::structs::types::RefOrPrim;

#[derive(Clone)]
pub struct Args {
    pub entries: Vec<RefOrPrim>,
}
