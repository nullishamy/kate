use crate::VMThread;
use std::collections::HashMap;

pub struct ThreadManager {}

impl ThreadManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_thread(&self, name: String) -> VMThread {
        VMThread::new(name)
    }
}
