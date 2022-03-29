use crate::classfile::parse_helper::SafeBuf;
use crate::runtime::bytecode::instruction::{Instruction, OperandType};
use crate::runtime::context::Context;
use crate::runtime::threading::thread::VMThread;
use crate::structs::loaded::default_attributes::AttributeEntry;
use crate::structs::loaded::method::MethodEntry;
use crate::{Level, SystemClassLoader};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use tracing::span;

//TODO: when we implement the move based loader system, consider how we will redefine the root loader (system)
pub struct VM {
    system_classloader: SystemClassLoader,
    interpreter_lock: Mutex<()>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            system_classloader: SystemClassLoader::new(),
            interpreter_lock: Mutex::new(()),
        }
    }

    pub fn interpret(&mut self, method: &MethodEntry, context: &mut Context) -> Result<()> {
        let can_run = self.interpreter_lock.lock();

        if let Err(err) = can_run {
            return Err(anyhow!(
                "thread panicked whilst waiting for the interpreter lock {}",
                err
            ));
        }

        let _raii = can_run.unwrap(); // not sure if this is needed. when this goes out of scope the mutex is released

        let instructions = method.attributes.get("Code");

        if instructions.is_none() {
            return Err(anyhow!(
                "method {} did not have a code attribute",
                method.name.as_str
            ));
        }

        let instructions = instructions.unwrap().as_code();

        if instructions.is_none() {
            return Err(anyhow!(
                "method {} did not have a code attribute",
                method.name.as_str
            ));
        }

        let instructions = instructions.unwrap();
        let mut bytes = Bytes::copy_from_slice(&instructions.code);

        while !bytes.is_empty() {
            let instruction = Instruction::from_byte(bytes.try_get_u8()?)?;

            match instruction {
                Instruction::AALOAD => todo!("unimplemented instruction "),
                Instruction::AASTORE => todo!("unimplemented instruction "),
                Instruction::ACONST_NULL => todo!("unimplemented instruction "),
                Instruction::ALOAD => todo!("unimplemented instruction "),
                Instruction::ALOAD_N => todo!("unimplemented instruction "),
                Instruction::ANEWARRAY => todo!("unimplemented instruction "),
                Instruction::ARETURN => todo!("unimplemented instruction "),
                Instruction::ARRAYLENGTH => todo!("unimplemented instruction "),
                Instruction::ASTORE => todo!("unimplemented instruction "),
                Instruction::ASTORE_N => todo!("unimplemented instruction "),
                Instruction::ATHROW => todo!("unimplemented instruction "),
                Instruction::BALOAD => todo!("unimplemented instruction "),
                Instruction::BASTORE => todo!("unimplemented instruction "),
                Instruction::BIPUSH => todo!("unimplemented instruction "),
                Instruction::CALOAD => todo!("unimplemented instruction "),
                Instruction::CASTORE => todo!("unimplemented instruction "),
                Instruction::CHECKCAST => todo!("unimplemented instruction "),
                Instruction::D2F => todo!("unimplemented instruction "),
                Instruction::D2I => todo!("unimplemented instruction "),
                Instruction::D2L => todo!("unimplemented instruction "),
                Instruction::DADD => todo!("unimplemented instruction "),
                Instruction::DALOAD => todo!("unimplemented instruction "),
                Instruction::DASTORE => todo!("unimplemented instruction "),
                Instruction::DCMPG => todo!("unimplemented instruction "),
                Instruction::DCMPL => todo!("unimplemented instruction "),
                Instruction::DCONST_N => todo!("unimplemented instruction "),
                Instruction::DDIV => todo!("unimplemented instruction "),
                Instruction::DLOAD => todo!("unimplemented instruction "),
                Instruction::DLOAD_N => todo!("unimplemented instruction "),
                Instruction::DMUL => todo!("unimplemented instruction "),
                Instruction::DNEG => todo!("unimplemented instruction "),
                Instruction::DREM => todo!("unimplemented instruction "),
                Instruction::DRETURN => todo!("unimplemented instruction "),
                Instruction::DSTORE => todo!("unimplemented instruction "),
                Instruction::DSTORE_N => todo!("unimplemented instruction "),
                Instruction::DSUB => todo!("unimplemented instruction "),
                Instruction::DUP => todo!("unimplemented instruction "),
                Instruction::DUP_X1 => todo!("unimplemented instruction "),
                Instruction::DUP_X2 => todo!("unimplemented instruction "),
                Instruction::DUP2 => todo!("unimplemented instruction "),
                Instruction::DUP2_X1 => todo!("unimplemented instruction "),
                Instruction::DUP2_X3 => todo!("unimplemented instruction "),
                Instruction::F2D => todo!("unimplemented instruction "),
                Instruction::F2I => todo!("unimplemented instruction "),
                Instruction::F2L => todo!("unimplemented instruction "),
                Instruction::FADD => todo!("unimplemented instruction "),
                Instruction::FALOAD => todo!("unimplemented instruction "),
                Instruction::FASTORE => todo!("unimplemented instruction "),
                Instruction::FCMPG => todo!("unimplemented instruction "),
                Instruction::FCMPL => todo!("unimplemented instruction "),
                Instruction::FCONST_N => todo!("unimplemented instruction "),
                Instruction::FDIV => todo!("unimplemented instruction "),
                Instruction::FLOAD => todo!("unimplemented instruction "),
                Instruction::FLOAD_N => todo!("unimplemented instruction "),
                Instruction::FMUL => todo!("unimplemented instruction "),
                Instruction::FNEG => todo!("unimplemented instruction "),
                Instruction::FREM => todo!("unimplemented instruction "),
                Instruction::FRETURN => todo!("unimplemented instruction "),
                Instruction::FSTORE => todo!("unimplemented instruction "),
                Instruction::FSTORE_N => todo!("unimplemented instruction "),
                Instruction::FSUB => todo!("unimplemented instruction "),
                Instruction::GETFIELD => todo!("unimplemented instruction "),
                Instruction::GETSTATIC => {
                    let idx = bytes.try_get_u16()?;
                    let field_ref = context.class.const_pool.field(idx as usize)?;

                    if field_ref.class.name.as_str == "java/lang/System"
                        && field_ref.name_and_type.name.as_str == "out"
                    {
                        let stack = context.thread.operand_stack.lock();

                        if let Err(err) = stack {
                            return Err(anyhow!(
                                "thread panicked whilst waiting for the stack lock {}",
                                err
                            ));
                        }

                        let mut stack = stack.unwrap();
                    }
                }
                Instruction::GOTO => todo!("unimplemented instruction "),
                Instruction::GOTO_W => todo!("unimplemented instruction "),
                Instruction::I2B => todo!("unimplemented instruction "),
                Instruction::I2C => todo!("unimplemented instruction "),
                Instruction::I2D => todo!("unimplemented instruction "),
                Instruction::I2F => todo!("unimplemented instruction "),
                Instruction::I2L => todo!("unimplemented instruction "),
                Instruction::I2S => todo!("unimplemented instruction "),
                Instruction::IADD => todo!("unimplemented instruction "),
                Instruction::IALOAD => todo!("unimplemented instruction "),
                Instruction::IAND => todo!("unimplemented instruction "),
                Instruction::IASTORE => todo!("unimplemented instruction "),
                Instruction::ICONST_N => todo!("unimplemented instruction "),
                Instruction::IDIV => todo!("unimplemented instruction "),
                Instruction::IF_ACMPEQ => todo!("unimplemented instruction "),
                Instruction::IF_ACMPNE => todo!("unimplemented instruction "),
                Instruction::IF_ICMPEQ => todo!("unimplemented instruction "),
                Instruction::IF_ICMPNE => todo!("unimplemented instruction "),
                Instruction::IF_ICMPLT => todo!("unimplemented instruction "),
                Instruction::IF_ICMPGT => todo!("unimplemented instruction "),
                Instruction::IF_ICMPLE => todo!("unimplemented instruction "),
                Instruction::IFEQ => todo!("unimplemented instruction "),
                Instruction::IFNE => todo!("unimplemented instruction "),
                Instruction::IFLT => todo!("unimplemented instruction "),
                Instruction::IFGE => todo!("unimplemented instruction "),
                Instruction::IFGT => todo!("unimplemented instruction "),
                Instruction::IFLE => todo!("unimplemented instruction "),
                Instruction::IFNOTNULL => todo!("unimplemented instruction "),
                Instruction::IFNULL => todo!("unimplemented instruction "),
                Instruction::IINC => todo!("unimplemented instruction "),
                Instruction::ILOAD => todo!("unimplemented instruction "),
                Instruction::ILOAD_N => todo!("unimplemented instruction "),
                Instruction::IMUL => todo!("unimplemented instruction "),
                Instruction::INEG => todo!("unimplemented instruction "),
                Instruction::INSTANCEOF => todo!("unimplemented instruction "),
                Instruction::INVOKEDYNAMIC => todo!("unimplemented instruction "),
                Instruction::INVOKEINTERFACE => todo!("unimplemented instruction "),
                Instruction::INVOKESPECIAL => todo!("unimplemented instruction "),
                Instruction::INVOKESTATIC => todo!("unimplemented instruction "),
                Instruction::INVOKEVIRTUAL => todo!("unimplemented instruction "),
                Instruction::IOR => todo!("unimplemented instruction "),
                Instruction::IREM => todo!("unimplemented instruction "),
                Instruction::IRETURN => todo!("unimplemented instruction "),
                Instruction::ISHL => todo!("unimplemented instruction "),
                Instruction::ISHR => todo!("unimplemented instruction "),
                Instruction::ISTORE => todo!("unimplemented instruction "),
                Instruction::ISTORE_N => todo!("unimplemented instruction "),
                Instruction::ISUB => todo!("unimplemented instruction "),
                Instruction::IUSHR => todo!("unimplemented instruction "),
                Instruction::IXOR => todo!("unimplemented instruction "),
                Instruction::JSR => todo!("unimplemented instruction "),
                Instruction::JSR_W => todo!("unimplemented instruction "),
                Instruction::L2D => todo!("unimplemented instruction "),
                Instruction::L2F => todo!("unimplemented instruction "),
                Instruction::L2I => todo!("unimplemented instruction "),
                Instruction::LADD => todo!("unimplemented instruction "),
                Instruction::LALOAD => todo!("unimplemented instruction "),
                Instruction::LAND => todo!("unimplemented instruction "),
                Instruction::LASTORE => todo!("unimplemented instruction "),
                Instruction::LCMP => todo!("unimplemented instruction "),
                Instruction::LCONST_N => todo!("unimplemented instruction "),
                Instruction::LMUL => todo!("unimplemented instruction "),
                Instruction::LNEG => todo!("unimplemented instruction "),
                Instruction::LOOKUPSWITCH => todo!("unimplemented instruction "),
                Instruction::LOR => todo!("unimplemented instruction "),
                Instruction::LREM => todo!("unimplemented instruction "),
                Instruction::LRETURN => todo!("unimplemented instruction "),
                Instruction::LSHL => todo!("unimplemented instruction "),
                Instruction::LSHR => todo!("unimplemented instruction "),
                Instruction::LSTORE => todo!("unimplemented instruction "),
                Instruction::LSTORE_N => todo!("unimplemented instruction "),
                Instruction::LSUB => todo!("unimplemented instruction "),
                Instruction::LUSHR => todo!("unimplemented instruction "),
                Instruction::LXOR => todo!("unimplemented instruction "),
                Instruction::MONITORENTER => todo!("unimplemented instruction "),
                Instruction::MONITOREXIT => todo!("unimplemented instruction "),
                Instruction::MULTIANEWARRAY => todo!("unimplemented instruction "),
                Instruction::NEW => todo!("unimplemented instruction "),
                Instruction::NEWARRAY => todo!("unimplemented instruction "),
                Instruction::NOP => todo!("unimplemented instruction "),
                Instruction::POP => todo!("unimplemented instruction "),
                Instruction::POP2 => todo!("unimplemented instruction "),
                Instruction::PUTFIELD => todo!("unimplemented instruction "),
                Instruction::PUTSTATIC => todo!("unimplemented instruction "),
                Instruction::RET => todo!("unimplemented instruction "),
                Instruction::RETURN => todo!("unimplemented instruction "),
                Instruction::SALOAD => todo!("unimplemented instruction "),
                Instruction::SASTORE => todo!("unimplemented instruction "),
                Instruction::SIPUSH => todo!("unimplemented instruction "),
                Instruction::SWAP => todo!("unimplemented instruction "),
                Instruction::TABLESWTICH => todo!("unimplemented instruction "),
                Instruction::WIDE => todo!("unimplemented instruction "),
            }
        }

        Ok(())
    }

    pub fn system_classloader(&self) -> &SystemClassLoader {
        &self.system_classloader
    }
}
