use parking_lot::{Mutex, RwLock};
use std::borrow::{Borrow, BorrowMut};
use std::sync::Arc;

use crate::runtime::heap::object::JVMObject;
use crate::runtime::stack::Stack;
use crate::runtime::threading::thread::{StackFrame, VMThread};
use crate::structs::loaded::method::MethodEntry;
use crate::LoadedClassFile;

#[derive(Debug, Clone)]
pub struct CallSite {
    pub class: Arc<LoadedClassFile>,
    pub thread: Arc<VMThread>,
    pub method: Arc<MethodEntry>,
    pub this_ref: Option<Arc<JVMObject>>,
    pub pc: Arc<RwLock<usize>>,
}

impl CallSite {
    pub fn new(
        class: Arc<LoadedClassFile>,
        thread: Arc<VMThread>,
        method: Arc<MethodEntry>,
        this_ref: Option<Arc<JVMObject>>,
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
