use std::sync::Arc;

use crate::structs::loaded::constant_pool::ClassData;

#[derive(Clone)]
pub struct Interfaces {
    pub entries: Vec<Arc<ClassData>>,
}
