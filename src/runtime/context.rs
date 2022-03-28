use crate::runtime::threading::thread::VMThread;
use crate::LoadedClassFile;
use std::rc::Rc;

pub struct Context {
    pub class: Rc<LoadedClassFile>,
    pub thread: VMThread,
}
