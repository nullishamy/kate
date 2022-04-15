use anyhow::{anyhow, Result};
use bytes::Bytes;
use parking_lot::RwLock;

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::bytecode::instruction_set::Instruction;
use crate::runtime::context::Context;
use crate::runtime::heap::Heap;
use crate::runtime::instruction::getstatic::get_static;
use crate::runtime::instruction::ldc::ldc;
use crate::runtime::threading::thread_manager::ThreadManager;
use crate::structs::loaded::method::MethodEntry;
use crate::{SystemClassLoader, TUIWriter, TuiCommand, VMConfig};

pub struct VM {
    pub system_classloader: RwLock<SystemClassLoader>,
    pub threads: RwLock<ThreadManager>,
    pub heap: RwLock<Heap>,
    pub tui: Option<TUIWriter>,

    state: VMState,
}

impl VM {
    pub fn new(config: VMConfig) -> Self {
        let mut s = Self {
            system_classloader: RwLock::new(SystemClassLoader::new()),
            threads: RwLock::new(ThreadManager::new()),
            heap: RwLock::new(Heap::new()),
            tui: config.tui,
            state: VMState::Shutdown,
        };

        s.state(VMState::Shutdown); // make sure we render this state, this should never fail
        s.state(VMState::Booting);

        s
    }

    pub fn state(&mut self, new_state: VMState) {
        self.state = new_state.clone();
        if let Some(t) = &self.tui {
            // sending stuff to TUI should never fail
            t.send(TuiCommand::VMState(new_state)).unwrap();
        }
    }

    pub fn interpret(&mut self, method: &MethodEntry, mut ctx: Context) -> Result<()> {
        self.state(VMState::Online);

        let instructions = method.attributes.get("Code");

        if instructions.is_none() {
            return Err(anyhow!(
                "method {} did not have a code attribute",
                method.name.str
            ));
        }

        let instructions = instructions.unwrap().as_code();

        if instructions.is_none() {
            return Err(anyhow!(
                "method {} did not have a code attribute",
                method.name.str
            ));
        }

        let instructions = instructions.unwrap();
        let mut instructions = Bytes::copy_from_slice(&instructions.code);

        while !instructions.is_empty() {
            let instruction = Instruction::from_byte(instructions.try_get_u8()?)?;

            match instruction {
                Instruction::LDC => ldc(self, &mut ctx, &mut instructions),
                Instruction::GETSTATIC => get_static(self, &mut ctx, &mut instructions),
                i => return Err(anyhow!("unimplemented instruction {:#?}", i)),
            }?
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum VMState {
    Shutdown,
    Booting,
    Online,
    Paused,
    ShuttingDown,
    GC,
}
