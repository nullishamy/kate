use crate::VMThread;


pub struct ThreadManager {}

impl ThreadManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_thread(&self, name: String) -> VMThread {
        VMThread::new(name)
    }
}
