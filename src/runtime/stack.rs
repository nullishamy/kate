use std::fmt::Debug;

use crate::structs::types::RefOrPrim;

#[derive(Debug, Clone)]
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
        self.items.push(value);
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

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }

    pub fn discard(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn flip(&mut self) {
        self.items.reverse();
    }

    pub fn raw(self) -> Vec<T> {
        self.items
    }

    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

pub type StackValue = RefOrPrim;
