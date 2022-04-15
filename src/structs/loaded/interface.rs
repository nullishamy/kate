use crate::structs::loaded::constant_pool::ClassData;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct Interfaces {
    pub entries: Vec<Arc<ClassData>>,
}
