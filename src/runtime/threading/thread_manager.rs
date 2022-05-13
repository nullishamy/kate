use crate::VmThread;

use crate::structs::loaded::method::MethodEntry;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ThreadManager {
    running_threads: HashMap<String, Arc<VmThread>>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            running_threads: HashMap::new(),
        }
    }

    pub fn new_thread(&mut self, name: String, code: Arc<MethodEntry>) -> Arc<VmThread> {
        let arc = Arc::new(VmThread::new(name.clone(), code));

        self.running_threads.insert(name, Arc::clone(&arc));

        arc
    }
}
