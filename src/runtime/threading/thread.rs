use crate::runtime::stack::{OperandType, Stack};
use crate::runtime::threading::result::ThreadResult;
use crate::runtime::threading::thread_manager::ThreadManager;
use crate::structs::types::{PrimitiveWithValue, Type};
use crate::structs::JVMPointer;
use parking_lot::Mutex;
use tokio::sync::oneshot;

pub struct VMThread {
    pub operand_stack: Mutex<Stack<OperandType>>,
    pub call_stack: Mutex<Stack<CallStackEntry>>,
    pub pc: usize,
    pub name: String,
}

impl VMThread {
    pub fn new(name: String) -> Self {
        Self {
            operand_stack: Mutex::new(Stack::new(0)),
            call_stack: Mutex::new(Stack::new(0)),
            pc: 0,
            name,
        }
    }

    // consume self, we can only run once
    pub fn run(self) -> oneshot::Receiver<ThreadResult> {
        let (send, recv) = oneshot::channel::<ThreadResult>();

        tokio::spawn(async move { send.send(Ok(Type::Primitive(PrimitiveWithValue::Int(1)))) });

        recv
    }
}

pub struct CallStackEntry {}
