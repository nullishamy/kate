use crate::structs::loaded::constant_pool::ClassData;
use std::rc::Rc;

#[derive(Clone)]
pub struct Interfaces {
    pub entries: Vec<Rc<ClassData>>,
}
