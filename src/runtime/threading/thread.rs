use parking_lot::Mutex;
use tokio::sync::oneshot;

use crate::runtime::stack::{Stack, StackValue};
use crate::runtime::threading::result::ThreadResult;
use crate::structs::types::{PrimitiveWithValue, RefOrPrim};

pub struct VMThread {
    pub operand_stack: Mutex<Stack<StackValue>>,
    pub locals: Mutex<Vec<StackValue>>,
    pub call_stack: Mutex<Stack<CallStackEntry>>,
    pub pc: usize,
    pub name: String,
}

impl VMThread {
    pub fn new(name: String) -> Self {
        Self {
            operand_stack: Mutex::new(Stack::new(0)),
            call_stack: Mutex::new(Stack::new(0)),
            locals: Mutex::new(Vec::new()),
            pc: 0,
            name,
        }
    }

    // consume self, we can only run once
    pub fn run(self) -> oneshot::Receiver<ThreadResult> {
        let (send, recv) = oneshot::channel::<ThreadResult>();

        //TODO: implement thread running
        tokio::spawn(
            async move { send.send(Ok(RefOrPrim::Primitive(PrimitiveWithValue::Int(1)))) },
        );

        recv
    }
}

#[derive(Clone, Debug)]
pub struct CallStackEntry {}
