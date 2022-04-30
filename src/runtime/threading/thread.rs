use std::sync::Arc;



use parking_lot::{Mutex};
use tokio::sync::oneshot;
use tokio::task::spawn_blocking;

use crate::{Args, CallSite, ClassLoader, VM};
use crate::runtime::heap::object::JVMObject;
use crate::runtime::stack::{Stack, StackValue};
use crate::runtime::threading::result::ThreadResult;
use crate::structs::loaded::method::MethodEntry;


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
    // this accepts a 'static VM so that we can be sure the ref
    // will live long enough and still exist when the closure is invoked
    // if i understand this correctly. idk it makes rustc happy lol.
    pub fn run(self: Arc<Self>, vm: &'static VM, args: Args) -> oneshot::Receiver<ThreadResult> {
        let (send, recv) = oneshot::channel::<ThreadResult>();

        let mut loader = vm.system_classloader.write();
        let thread_class = loader.load_class("java/lang/Thread").unwrap();

        let this = Arc::new(JVMObject {
            class: Arc::clone(&thread_class),
        });
        
        let callsite = CallSite::new(thread_class, Arc::clone(&self), Arc::clone(&self.method), Some(this));

        //TODO: change this to async once we implement async interpretation
        spawn_blocking(|| {
            // any blocking operations will get transformed into async ones here, hopefully
            // in order for this to work, the entire interpreter needs to be async
            // which is a long way off. for now, this will just be blocking
            let res = vm.interpret(callsite, args,false );
            send.send(res)
        });

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
