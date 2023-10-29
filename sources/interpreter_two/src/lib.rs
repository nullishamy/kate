use anyhow::Result;
use bytecode::decode_instruction;
use bytes::BytesMut;

use object::{
    classloader::ClassLoader, statics::StaticFields, string::Interner, RuntimeValue,
    WrappedClassObject,
};
use parse::attributes::CodeAttribute;
use tracing::info;

use crate::{native::NativeModule, object::statics::StaticFieldRef};

pub mod bytecode;
pub mod error;
pub mod native;
pub mod object;

pub struct Context {
    pub code: CodeAttribute,
    pub class: WrappedClassObject,

    pub pc: i32,
    pub operands: Vec<RuntimeValue>,
    pub locals: Vec<RuntimeValue>,
}

pub struct VM {
    pub class_loader: ClassLoader,
    pub interner: Interner,
    pub statics: StaticFields,
}

impl VM {
    pub fn run(&mut self, mut ctx: Context) -> Result<Option<RuntimeValue>> {
        while ctx.pc < ctx.code.code.len() as i32 {
            let slice = &ctx.code.code[ctx.pc as usize..];
            let consumed_bytes_prev = slice.len();

            let mut code_bytes = BytesMut::new();
            code_bytes.extend_from_slice(slice);

            let mut instruction_bytes = BytesMut::new();
            instruction_bytes.extend_from_slice(slice);

            let instruction = decode_instruction(self, &mut instruction_bytes)?;
            let consumed_bytes_post = instruction_bytes.len();
            let bytes_consumed_by_opcode = (consumed_bytes_prev - consumed_bytes_post) as i32;
            info!(
                "opcode: {:?} consumed {} bytes",
                instruction, bytes_consumed_by_opcode
            );

            match instruction.handle(self, &mut ctx)? {
                bytecode::Progression::JumpAbs(new_pc) => {
                    info!("Jumping from {} to {}", ctx.pc, new_pc);
                    ctx.pc = new_pc;
                }
                bytecode::Progression::JumpRel(offset) => {
                    info!(
                        "Jumping from {} by {} (new: {})",
                        ctx.pc,
                        offset,
                        ctx.pc + offset
                    );
                    ctx.pc += offset;
                }
                bytecode::Progression::Next => {
                    info!(
                        "Moving to next (jump by {} bytes)",
                        bytes_consumed_by_opcode
                    );
                    ctx.pc += bytes_consumed_by_opcode;
                }
                bytecode::Progression::Return(return_value) => {
                    info!("Returning");
                    return Ok(return_value);
                }
                bytecode::Progression::Throw(err) => {
                    info!("Throwing {:?}", err);
                    return Err(err);
                }
            };
        }

        Ok(None)
    }

    pub fn initialise_class(&mut self, class: WrappedClassObject) -> Result<()> {
        let mut locked_class = class.write();
        let class_name = locked_class.get_class_name();

        if locked_class.is_initialised() {
            info!(
                "Not initialising {}, class is already initialised",
                class_name
            );

            return Ok(());
        }

        let clinit = locked_class.get_method(("<clinit>".to_string(), "()V".to_string()));

        if let Some(clinit) = clinit {
            info!("Initialising {}", class_name);

            // Need to drop our lock on the class object before running the class initialiser
            // as it could call instructions which access class data
            locked_class.set_initialised(true);
            let code = clinit
                .attributes
                .known_attribute(locked_class.constant_pool())?;
            drop(locked_class);

            let ctx = Context {
                code,
                class,
                pc: 0,
                operands: vec![],
                locals: vec![],
            };

            self.run(ctx)?;
        } else {
            info!("No clinit in {}", class_name);
            // Might as well mark this as initialised to avoid future
            // needless method lookups
            locked_class.set_initialised(true);
        }

        Ok(())
    }

    pub fn bootstrap(&mut self) -> Result<()> {
        // Load native modules
        use native::lang;
        lang::Class::register(self)?;
        lang::Throwable::register(self)?;
        lang::StringUTF16::register(self)?;
        lang::Float::register(self)?;
        lang::Double::register(self)?;
        lang::System::register(self)?;

        self.statics.set_field(
            StaticFieldRef::new_str("java/lang/String", "COMPACT_STRINGS", "Z"),
            RuntimeValue::Integral(0_i32.into()),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
