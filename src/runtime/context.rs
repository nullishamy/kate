use std::sync::Arc;

use crate::runtime::threading::thread::VMThread;
use crate::LoadedClassFile;

pub struct Context {
    pub class: Arc<LoadedClassFile>,
    pub thread: Arc<VMThread>,
}
