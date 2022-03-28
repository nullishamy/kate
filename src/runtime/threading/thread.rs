use crate::runtime::bytecode::instruction::OperandType;
use crate::runtime::stack::Stack;
use std::sync::Mutex;

pub struct VMThread {
    pub operand_stack: Mutex<Stack<OperandType>>,
    pub call_stack: Mutex<Stack<CallStackEntry>>,
}

impl VMThread {
    pub fn new() -> Self {
        Self {
            operand_stack: Mutex::new(Stack::new(0)),
            call_stack: Mutex::new(Stack::new(0)),
        }
    }
}

pub struct CallStackEntry {}
