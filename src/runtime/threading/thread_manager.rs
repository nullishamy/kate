use crate::VMThread;

use std::collections::HashMap;
use std::sync::Arc;
use crate::structs::loaded::method::MethodEntry;

pub struct ThreadManager {
    running_threads: HashMap<String, Arc<VMThread>>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            running_threads: HashMap::new(),
        }
    }

    pub fn new_thread(&mut self, name: String, code: Arc<MethodEntry>) -> Arc<VMThread> {
        let arc = Arc::new(VMThread::new(name.clone(), code));

        self.running_threads.insert(name, Arc::clone(&arc));

        arc
    }
}
