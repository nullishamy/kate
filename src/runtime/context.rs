use std::sync::Arc;

use crate::runtime::threading::thread::VMThread;
use crate::LoadedClassFile;

#[derive(Clone)]
pub struct Context {
    pub class: Arc<LoadedClassFile>,
    pub thread: Arc<VMThread>,
}
