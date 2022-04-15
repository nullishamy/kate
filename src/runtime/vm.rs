use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::bytecode::instruction_set::Instruction;
use crate::runtime::context::Context;
use crate::runtime::heap::Heap;
use crate::runtime::stack::OperandType;
use std::borrow::Borrow;

use crate::runtime::instruction::getstatic::get_static;
use crate::runtime::instruction::ldc::ldc;

use crate::runtime::threading::thread_manager::ThreadManager;
use crate::structs::loaded::constructors::Constructor;
use crate::structs::loaded::method::MethodEntry;
use crate::structs::types::{Int, PrimitiveType, PrimitiveWithValue};
use crate::{ClassLoader, SystemClassLoader};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use parking_lot::{Mutex, RwLock};
use std::rc::Rc;
use tracing::debug;
use tracing::field::debug;

pub struct VM {
    pub system_classloader: RwLock<SystemClassLoader>,
    pub threads: RwLock<ThreadManager>,
    pub heap: RwLock<Heap>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            system_classloader: RwLock::new(SystemClassLoader::new()),
            threads: RwLock::new(ThreadManager::new()),
            heap: RwLock::new(Heap::new()),
        }
    }

    pub fn interpret(&mut self, method: &MethodEntry, mut ctx: Context) -> Result<()> {
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
                i => todo!("unimplemented instruction {:#?}", i),
            }?
        }

        Ok(())
    }
}
