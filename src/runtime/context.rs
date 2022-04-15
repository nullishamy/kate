use crate::runtime::threading::thread::VMThread;
use crate::LoadedClassFile;
use std::rc::Rc;
use std::sync::Arc;

pub struct Context {
    pub class: Arc<LoadedClassFile>,
    pub thread: VMThread,
}
