use crate::VMThread;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ThreadManager {
    running_threads: HashMap<String, Arc<VMThread>>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            running_threads: HashMap::new(),
        }
    }

    pub fn new_thread(&mut self, name: String) -> Arc<VMThread> {
        let arc = Arc::new(VMThread::new(name.clone()));
        self.running_threads.insert(name, Arc::clone(&arc));

        arc
    }
}
