use anyhow::{anyhow, Result};
use bytes::Bytes;
use parking_lot::RwLock;
use std::process::exit;
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::bytecode::instruction_set::Instruction;
use crate::runtime::callsite::CallSite;
use crate::runtime::heap::Heap;
use crate::runtime::instruction::_return::_return;
use crate::runtime::instruction::aconst_null::aconst_null;
use crate::runtime::instruction::aload::aload;
use crate::runtime::instruction::areturn::areturn;
use crate::runtime::instruction::astore::astore;
use crate::runtime::instruction::dup::dup;
use crate::runtime::instruction::getstatic::get_static;
use crate::runtime::instruction::iconst::iconst;
use crate::runtime::instruction::ifnull::ifnull;
use crate::runtime::instruction::iload::iload;
use crate::runtime::instruction::invokespecial::invoke_special;
use crate::runtime::instruction::invokestatic::invoke_static;
use crate::runtime::instruction::invokevirtual::invoke_virtual;
use crate::runtime::instruction::ldc::ldc;
use crate::runtime::instruction::new::new;
use crate::runtime::instruction::putstatic::put_static;
use crate::runtime::threading::thread::StackFrame;
use crate::runtime::threading::thread_manager::ThreadManager;

use crate::runtime::bytecode::args::Args;
use crate::runtime::instruction::bipush::bipush;
use crate::runtime::native::method_controller::NativeMethodController;
use crate::runtime::stack::StackValue;
use crate::structs::types::{RefOrPrim, ReferenceType};
use crate::{MethodAccessFlag, SystemClassLoader, TUIWriter, TuiCommand, VMConfig, VMThread};

// this struct is entirely interior mutable so that we can pass it around freely
pub struct VM {
    pub system_classloader: RwLock<SystemClassLoader>,
    pub threads: RwLock<ThreadManager>,
    pub heap: RwLock<Heap>,
    pub tui: Option<TUIWriter>,
    pub native: RwLock<NativeMethodController>,

    state: RwLock<VMState>,
}

impl VM {
    pub fn new(config: VMConfig) -> Self {
        let s = Self {
            system_classloader: RwLock::new(SystemClassLoader::new()),
            threads: RwLock::new(ThreadManager::new()),
            heap: RwLock::new(Heap::new(config.tui.as_ref().cloned())),
            tui: config.tui,
            state: RwLock::new(VMState::Shutdown),
            native: RwLock::new(NativeMethodController::new()),
        };

        s.state(VMState::Shutdown); // make sure we render this state, this should never fail
        s.state(VMState::Booting);

        s
    }

    pub fn state(&self, new_state: VMState) {
        *self.state.write() = new_state.clone();
        if let Some(t) = &self.tui {
            // sending stuff to TUI should never fail
            // if it does, the TUI task has panicked and we should too
            t.send(TuiCommand::VMState(new_state)).unwrap();
        }
    }

    pub fn interpret(
        &self,
        mut callsite: CallSite,
        mut args: Args,
        continuation: bool,
    ) -> Result<()> {
        let clone = callsite.clone();

        let method = clone.method;
        let caller_class = clone.class;
        let caller_thread = clone.thread;
        let this_ref = clone.this_ref;

        if continuation {
            debug!(
                "continuing execution of method {} in class {} on thread {}",
                method.name.str, caller_class.this_class.name.str, caller_thread.name
            );
        } else {
            debug!(
                "interpreting method {} in class {} on thread {}",
                method.name.str, caller_class.this_class.name.str, caller_thread.name
            );
        }

        //TODO: stackframe support for native functions
        if method.access_flags.has(MethodAccessFlag::NATIVE) {
            // f/q/c/n.methodName:(descriptor)
            let name = format!(
                "{}.{}:{}",
                caller_class.this_class.name.str,
                method.name.str,
                method.descriptor.to_string()
            );

            let method = self.native.read();
            let method = method.entries.get(&name);

            if method.is_none() {
                error!("method {} was not registered with the native manager", name);
                exit(1);
            }

            // we return here because native stuff should not interfere with VM stuff below
            // native functions could call VM functions and this would be skipped
            return method.unwrap()(self, &mut args, &mut callsite);
        }

        let mut call_stack = caller_thread.call_stack.lock();

        // if we arent continuing execution from somewhere, push a new frame
        if !continuation {
            let mut f = StackFrame::new(callsite.clone());

            if let Some(this_ref) = this_ref {
                // push thisref as the first local if it exists. it might not (statics)
                f.locals
                    .push(RefOrPrim::Reference(ReferenceType::Class(this_ref)))
            }

            while let Some(arg) = args.entries.pop() {
                f.locals.push(arg); // push all the args into locals
            }

            call_stack.push(f);
            debug!("pushed a new stack frame to the call stack");
        }

        drop(call_stack);

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
                "method {} did not have a valid code attribute",
                method.name.str
            ));
        }

        let instructions = instructions.unwrap();

        // read from pc => end
        // this is more efficient and allows us to jump
        // pc will be reset by return statements
        let instructions = &instructions.code[*callsite.pc.read()..];

        let mut bytes = Bytes::copy_from_slice(instructions);

        while !bytes.is_empty() {
            let instruction = Instruction::from_byte(bytes.try_get_u8()?)?;

            debug!("interpreting {:?}", instruction);

            let res = match instruction {
                Instruction::LDC => ldc(self, &mut callsite, &mut bytes),
                Instruction::GETSTATIC => get_static(self, &mut callsite, &mut bytes),

                Instruction::ALOAD_0 => aload(self, &mut callsite, 0),
                Instruction::ALOAD_1 => aload(self, &mut callsite, 1),
                Instruction::ALOAD_2 => aload(self, &mut callsite, 2),
                Instruction::ALOAD_3 => aload(self, &mut callsite, 3),
                Instruction::ALOAD => aload(self, &mut callsite, bytes.try_get_u16()?),

                Instruction::ASTORE_0 => astore(self, &mut callsite, 0),
                Instruction::ASTORE_1 => astore(self, &mut callsite, 1),
                Instruction::ASTORE_2 => astore(self, &mut callsite, 2),
                Instruction::ASTORE_3 => astore(self, &mut callsite, 3),
                Instruction::ASTORE => astore(self, &mut callsite, bytes.try_get_u16()?),

                Instruction::ICONST_M1 => iconst(self, &mut callsite, -1),
                Instruction::ICONST_0 => iconst(self, &mut callsite, 0),
                Instruction::ICONST_1 => iconst(self, &mut callsite, 1),
                Instruction::ICONST_2 => iconst(self, &mut callsite, 2),
                Instruction::ICONST_3 => iconst(self, &mut callsite, 3),
                Instruction::ICONST_4 => iconst(self, &mut callsite, 4),
                Instruction::ICONST_5 => iconst(self, &mut callsite, 5),

                Instruction::INVOKESTATIC => invoke_static(self, &mut callsite, &mut bytes),
                Instruction::INVOKESPECIAL => invoke_special(self, &mut callsite, &mut bytes),
                Instruction::INVOKEVIRTUAL => invoke_virtual(self, &mut callsite, &mut bytes),

                Instruction::ACONST_NULL => aconst_null(self, &mut callsite, &mut bytes),
                Instruction::PUTSTATIC => put_static(self, &mut callsite, &mut bytes),
                Instruction::RETURN => _return(self, &mut callsite, &mut bytes),

                Instruction::NEW => new(self, &mut callsite, &mut bytes),
                Instruction::DUP => dup(self, &mut callsite, &mut bytes),
                Instruction::ARETURN => areturn(self, &mut callsite, &mut bytes),
                // args is needed for continuation here
                Instruction::IFNULL => ifnull(self, &mut callsite, &mut args, &mut bytes),
                Instruction::BIPUSH => bipush(self, &mut callsite, &mut bytes),

                Instruction::ILOAD_0 => iload(self, &mut callsite, 0),
                Instruction::ILOAD_1 => iload(self, &mut callsite, 1),
                Instruction::ILOAD_2 => iload(self, &mut callsite, 2),
                Instruction::ILOAD_3 => iload(self, &mut callsite, 3),
                Instruction::ILOAD => iload(self, &mut callsite, bytes.try_get_u16()?),

                i => Err(anyhow!("unimplemented instruction {:#?}", i)),
            };

            // pc will have whichever instruction we are currently operating on
            *callsite.pc.write() += 2;

            if let Err(e) = res {
                error!("interpreter returned error `{e}`");
                exit(1)
            }
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
