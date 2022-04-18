use anyhow::{anyhow, Result};
use bytes::Bytes;
use parking_lot::RwLock;
use tracing::{debug, warn};

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::bytecode::instruction_set::Instruction;
use crate::runtime::context::Context;
use crate::runtime::heap::Heap;
use crate::runtime::instruction::_return::_return;
use crate::runtime::instruction::aconst_null::aconst_null;
use crate::runtime::instruction::aload::aload;
use crate::runtime::instruction::dup::dup;
use crate::runtime::instruction::getstatic::get_static;
use crate::runtime::instruction::iconst::iconst;
use crate::runtime::instruction::invokestatic::invoke_static;
use crate::runtime::instruction::ldc::ldc;
use crate::runtime::instruction::new::new;
use crate::runtime::instruction::putstatic::put_static;
use crate::runtime::threading::thread::CallStackEntry;
use crate::runtime::threading::thread_manager::ThreadManager;
use crate::structs::loaded::method::MethodEntry;
use crate::{MethodAccessFlag, SystemClassLoader, TUIWriter, TuiCommand, VMConfig};

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
            heap: RwLock::new(Heap::new(config.tui.as_ref().cloned())),
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
            // if it does, the TUI task has panicked and we should too
            t.send(TuiCommand::VMState(new_state)).unwrap();
        }
    }

    pub fn interpret(&mut self, method: &MethodEntry, mut ctx: Context) -> Result<()> {
        debug!(
            "interpreting method {} in class {} on thread {}",
            method.name.str, ctx.class.this_class.name.str, ctx.thread.name
        );

        self.state(VMState::Online);

        ctx.thread.call_stack.lock().push(CallStackEntry {});

        if method.access_flags.has(MethodAccessFlag::NATIVE) {
            warn!(
                "attempted to call native method {}, which is not implemented",
                method.name.str
            );
            return Ok(());
        }

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
                "method {} did not have a valid code attribute",
                method.name.str
            ));
        }

        let instructions = instructions.unwrap();
        let mut bytes = Bytes::copy_from_slice(&instructions.code);

        while !bytes.is_empty() {
            let instruction = Instruction::from_byte(bytes.try_get_u8()?)?;

            match instruction {
                Instruction::LDC => ldc(self, &mut ctx, &mut bytes),
                Instruction::GETSTATIC => get_static(self, &mut ctx, &mut bytes),

                Instruction::ALOAD_0 => aload(self, &mut ctx, 0),
                Instruction::ALOAD_1 => aload(self, &mut ctx, 1),
                Instruction::ALOAD_2 => aload(self, &mut ctx, 2),
                Instruction::ALOAD_3 => aload(self, &mut ctx, 3),
                Instruction::ALOAD => aload(self, &mut ctx, bytes.try_get_u16()?),

                Instruction::ICONST_M1 => iconst(self, &mut ctx, -1),
                Instruction::ICONST_0 => iconst(self, &mut ctx, 0),
                Instruction::ICONST_1 => iconst(self, &mut ctx, 1),
                Instruction::ICONST_2 => iconst(self, &mut ctx, 2),
                Instruction::ICONST_3 => iconst(self, &mut ctx, 3),
                Instruction::ICONST_4 => iconst(self, &mut ctx, 4),
                Instruction::ICONST_5 => iconst(self, &mut ctx, 5),

                Instruction::INVOKESTATIC => invoke_static(self, &mut ctx, &mut bytes),
                Instruction::ACONST_NULL => aconst_null(self, &mut ctx, &mut bytes),
                Instruction::PUTSTATIC => put_static(self, &mut ctx, &mut bytes),
                Instruction::RETURN => _return(self, &mut ctx, &mut bytes),

                Instruction::NEW => new(self, &mut ctx, &mut bytes),
                Instruction::DUP => dup(self, &mut ctx, &mut bytes),
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
