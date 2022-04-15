use crate::runtime::heap::object::JVMObject;
use crate::structs::types::PrimitiveWithValue;


use std::sync::Arc;

pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn push(&mut self, value: T) {
        self.items.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }
}

pub enum OperandType {
    Primitive(PrimitiveWithValue),
    Reference(Arc<JVMObject>),
}
