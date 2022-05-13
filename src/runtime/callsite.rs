use parking_lot::RwLock;

use std::sync::Arc;

use crate::runtime::heap::object::JvmObject;

use crate::runtime::threading::thread::VmThread;
use crate::structs::loaded::method::MethodEntry;
use crate::LoadedClassFile;

#[derive(Debug, Clone)]
pub struct CallSite {
    pub class: Arc<LoadedClassFile>,
    pub thread: Arc<VmThread>,
    pub method: Arc<MethodEntry>,
    pub this_ref: Option<Arc<JvmObject>>,
    pub pc: Arc<RwLock<usize>>,
}

impl CallSite {
    pub fn new(
        class: Arc<LoadedClassFile>,
        thread: Arc<VmThread>,
        method: Arc<MethodEntry>,
        this_ref: Option<Arc<JvmObject>>,
    ) -> Self {
        Self {
            class,
            thread,
            method,
            this_ref,
            pc: Arc::new(RwLock::new(0)),
        }
    }
}
