use parking_lot::{Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::runtime::heap::object::JVMObject;
use crate::runtime::stack::{Stack, StackValue};
use crate::runtime::threading::result::ThreadResult;
use crate::structs::loaded::method::MethodEntry;
use crate::structs::types::{PrimitiveWithValue, RefOrPrim};
use crate::{Args, CallSite, ClassLoader, LoadedClassFile, VM};

#[derive(Debug)]
pub struct VMThread {
    pub name: String,
    pub call_stack: Mutex<Stack<StackFrame>>,

    method: Arc<MethodEntry>,
}

impl VMThread {
    pub fn new(name: String, method: Arc<MethodEntry>) -> Self {
        Self {
            name,
            call_stack: Mutex::new(Stack::new()),
            method,
        }
    }

    // consume self, we can only run once
    pub fn run(self: Arc<Self>, vm: &VM, args: Args) -> oneshot::Receiver<ThreadResult> {
        let (send, recv) = oneshot::channel::<ThreadResult>();

        let mut loader = vm.system_classloader.write();
        let thread_class = loader.load_class("java/lang/Thread").unwrap();

        let this = JVMObject {
            class: Arc::clone(&thread_class),
        };

        //TODO: implement thread running
        let callsite = CallSite::new(thread_class, self, self.method, this);
        tokio::spawn(async move { send.send() });

        recv
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub operand_stack: Stack<StackValue>,
    pub locals: Vec<StackValue>,
    pub callsite: CallSite,
}

impl StackFrame {
    pub fn new(callsite: CallSite) -> Self {
        //FIXME: use attrs to determine these
        Self {
            operand_stack: Stack::new(),
            locals: Vec::with_capacity(30),
            callsite,
        }
    }
}
