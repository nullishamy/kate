use anyhow::Result;
use bytecode::decode_instruction;
use bytes::BytesMut;

use object::{classloader::ClassLoader, RuntimeValue, WrappedClassObject, string::Interner};
use parse::attributes::CodeAttribute;
use tracing::info;

pub mod bytecode;
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
    pub interner: Interner
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
            info!("opcode: {:?} consumed {} bytes", instruction, bytes_consumed_by_opcode);

            // If the instruction doesn't want us to jump anywhere, proceed to the next instruction
            match  instruction.handle(self, &mut ctx)? {
                bytecode::Progression::JumpAbs(new_pc) => {
                    info!("Jumping from {} to {}", ctx.pc, new_pc);
                    ctx.pc = new_pc;
                },
                bytecode::Progression::JumpRel(offset) => {
                    info!("Jumping from {} by {} (new: {})", ctx.pc, offset, ctx.pc + offset);
                    ctx.pc += offset;
                },
                bytecode::Progression::Next => {
                    info!("Moving to next (jump by {} bytes)", bytes_consumed_by_opcode);
                    ctx.pc += bytes_consumed_by_opcode;
                },
                bytecode::Progression::Return(return_value) => {
                    info!("Returning");
                    return Ok(return_value)
                },
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
            let code = clinit.attributes.known_attribute(locked_class.constant_pool())?;
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
}

#[cfg(test)]
mod tests {
    use parse::attributes::CodeAttribute;

    use crate::{object::{classloader::ClassLoader, string::Interner}, Context, VM};

    #[test]
    fn it_runs_empty_main_functions() {
        let source_root = env!("CARGO_MANIFEST_DIR");
        let mut class_loader = ClassLoader::new();

        class_loader
            .add_path(format!("{source_root}/../../std/java.base").into())
            .add_path(format!("{source_root}/../../samples").into());

        let (_, jls) = class_loader.bootstrap().unwrap();

        let mut vm = VM {
            class_loader,
            interner: Interner::new(jls)
        };


        let _cls = vm.class_loader.load_class("JustMain".to_string()).unwrap();
        let cls = _cls.read();

        let method = cls
            .get_method(("main".to_string(), "([Ljava/lang/String;)V".to_string()))
            .unwrap();

        let code = method
            .attributes
            .known_attribute::<CodeAttribute>(cls.constant_pool())
            .unwrap();

        drop(cls);

        let executable = Context {
            code,
            operands: vec![],
            locals: vec![],
            pc: 0,
            class: _cls
        };

        vm.run(executable).unwrap();
    }
}
