use std::fmt::Debug;

use crate::structs::types::RefOrPrim;

#[derive(Debug)]
pub struct Stack<T>
where
    T: Debug,
{
    items: Vec<T>,
}

impl<T> Stack<T>
where
    T: Debug,
{
    pub fn push(&mut self, value: T) {
        self.items.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn discard(&mut self) {
        self.items.clear()
    }

    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }
}

pub type StackValue = RefOrPrim;
