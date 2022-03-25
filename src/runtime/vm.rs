use crate::SystemClassLoader;

pub struct VM {
    pub system_classloader: SystemClassLoader,
}

impl VM {
    pub fn new() -> Self {
        Self {
            system_classloader: SystemClassLoader::new(),
        }
    }
}
