#![feature(pointer_byte_offsets)]
#![feature(offset_of)]

use bytecode::decode_instruction;
use bytes::BytesMut;

use error::{Frame, Throwable, VMError};
use object::{
    builtins::{BuiltinString, Class, Object},
    loader::ClassLoader,
    mem::RefTo,
    runtime::RuntimeValue,
};
use parse::attributes::CodeAttribute;

use tracing::{debug, info, trace};

use crate::object::{
    builtins::Array,
    interner::intern_string,
    layout::types::{Bool, Int, Long},
    mem::HasObjectHeader,
};

pub mod bytecode;
pub mod error;
pub mod native;
pub mod object;

pub struct Context {
    pub code: CodeAttribute,
    pub class: RefTo<Class>,

    pub pc: i32,
    pub operands: Vec<RuntimeValue>,
    pub locals: Vec<RuntimeValue>,
}

pub struct VM {
    pub class_loader: ClassLoader,
    pub frames: Vec<Frame>,

    pub main_thread: RefTo<Object>,
}

#[derive(Debug)]
pub struct ThrownState {
    pub pc: i32,
}

impl VM {
    pub fn run(
        &mut self,
        mut ctx: Context,
    ) -> Result<Option<RuntimeValue>, (Throwable, ThrownState)> {
        while ctx.pc < ctx.code.code.len() as i32 {
            let slice = &ctx.code.code[ctx.pc as usize..];
            let consumed_bytes_prev = slice.len();

            let mut code_bytes = BytesMut::new();
            code_bytes.extend_from_slice(slice);

            let mut instruction_bytes = BytesMut::new();
            instruction_bytes.extend_from_slice(slice);

            let instruction = decode_instruction(self, &mut instruction_bytes, &ctx)
                .map_err(|e| (e, ThrownState { pc: ctx.pc }))?;

            let consumed_bytes_post = instruction_bytes.len();
            let bytes_consumed_by_opcode = (consumed_bytes_prev - consumed_bytes_post) as i32;
            trace!(
                "opcode: {:?} consumed {} bytes",
                instruction,
                bytes_consumed_by_opcode
            );

            let progression = instruction
                .handle(self, &mut ctx)
                .map_err(|e| (e, ThrownState { pc: ctx.pc }))?;

            match progression {
                bytecode::Progression::JumpAbs(new_pc) => {
                    debug!("Jumping from {} to {}", ctx.pc, new_pc);
                    ctx.pc = new_pc;
                }
                bytecode::Progression::JumpRel(offset) => {
                    debug!(
                        "Jumping from {} by {} (new: {})",
                        ctx.pc,
                        offset,
                        ctx.pc + offset
                    );
                    ctx.pc += offset;
                }
                bytecode::Progression::Next => {
                    debug!(
                        "Moving to next (jump by {} bytes)",
                        bytes_consumed_by_opcode
                    );
                    ctx.pc += bytes_consumed_by_opcode;
                }
                bytecode::Progression::Return(return_value) => {
                    debug!("Returning");
                    return Ok(return_value);
                }
                bytecode::Progression::Throw(err) => {
                    info!("Throwing {}", err);
                    return Err((err, ThrownState { pc: ctx.pc }));
                }
            };
        }

        Ok(None)
    }

    pub fn initialise_class(&mut self, class: RefTo<Class>) -> Result<(), Throwable> {
        let class_name = class.borrow().name().clone();

        if class.borrow().is_initialised() {
            debug!(
                "Not initialising {}, class is already initialised",
                class_name
            );

            return Ok(());
        }

        let clinit = class
            .borrow()
            .class_file()
            .methods
            .locate("<clinit>".to_string(), "()V".to_string())
            .cloned();

        if let Some(clinit) = clinit {
            debug!("Initialising {}", class_name);

            // Need to drop our lock on the class object before running the class initialiser
            // as it could call instructions which access class data
            class.borrow_mut().set_initialised(true);
            let code = clinit
                .attributes
                .known_attribute(&class.borrow().class_file().constant_pool)?;

            let ctx = Context {
                code,
                class,
                pc: 0,
                operands: vec![],
                locals: vec![],
            };

            // TODO: Handle this properly, as clinit errors should be caught
            self.run(ctx).map_err(|e| e.0)?;
            debug!("Finished initialising {}", class_name);
        } else {
            debug!("No clinit in {}", class_name);
            // Might as well mark this as initialised to avoid future
            // needless method lookups
            class.borrow_mut().set_initialised(true);
        }

        Ok(())
    }

    pub fn main_thread(&self) -> RefTo<Object> {
        self.main_thread.clone()
    }

    pub fn make_error(&mut self, ty: VMError) -> Result<Throwable, Throwable> {
        match ty {
            VMError::ArrayIndexOutOfBounds { .. } => {
                let cls = self.class_loader.for_name(ty.class_name().to_string())?;
                Ok(Throwable::Runtime(error::RuntimeException {
                    message: ty.message(),
                    ty: cls,
                    obj: RuntimeValue::null_ref(),
                    sources: self.frames.clone(),
                }))
            }
        }
    }

    pub fn bootstrap(&mut self) -> Result<(), Throwable> {
        // Load native modules
        use native::*;
        lang::LangClass::register(self)?;
        lang::Throwable::register(self)?;
        lang::StringUTF16::register(self)?;
        lang::Float::register(self)?;
        lang::LangDouble::register(self)?;
        lang::System::register(self)?;
        lang::LangObject::register(self)?;
        lang::Runtime::register(self)?;
        lang::LangThread::register(self)?;

        jdk::Cds::register(self)?;
        jdk::Reflection::register(self)?;
        jdk::SystemPropsRaw::register(self)?;
        jdk::Unsafe::register(self)?;
        jdk::JdkVm::register(self)?;
        jdk::ScopedMemoryAccess::register(self)?;
        jdk::Signal::register(self)?;

        io::FileDescriptor::register(self)?;
        io::FileOutputStream::register(self)?;

        let jls = self.class_loader.for_name("java/lang/String".to_string())?;
        // Init String so that we can set the static after it's done. The clinit sets it to a default.
        self.initialise_class(jls.clone())?;

        jls.borrow_mut()
            .static_field_info_mut(("COMPACT_STRINGS".to_string(), "Z".to_string()))
            .unwrap()
            .value = Some(RuntimeValue::Integral(0_i32.into()));

        // Init thread
        let thread_class = self.class_loader.for_name("java/lang/Thread".to_string())?;
        // self.initialise_class(thread_class.clone())?;

        let thread_group_class = self
            .class_loader
            .for_name("java/lang/ThreadGroup".to_string())?;
        // self.initialise_class(thread_class.clone())?;

        let thread = BuiltinThread {
            object: Object {
                class: thread_class.clone(),
                super_class: thread_class.borrow().super_class(),
                ref_count: 0,
            },
            name: intern_string("main".to_string())?,
            priority: 0,
            daemon: 0,
            interrupted: 0,
            stillborn: 0,
            eetop: 0,
            target: RefTo::null(),
            thread_group: RefTo::new(BuiltinThreadGroup {
                object: Object {
                    class: thread_group_class.clone(),
                    super_class: thread_group_class.borrow().super_class(),
                    ref_count: 0,
                },
                parent: RefTo::null(),
                name: intern_string("main".to_string())?,
                max_priority: 0,
                destroyed: 0,
                daemon: 0,
                n_unstarted_threads: 0,
                n_threads: 0,
                threads: RefTo::null(),
                n_groups: 0,
                groups: RefTo::null(),
            }),
            context_class_loader: RefTo::null(),
            inherited_access_control_context: RefTo::null(),
            thread_locals: RefTo::null(),
            inheritable_thread_locals: RefTo::null(),
            stack_size: 0,
            tid: 1,
            status: 1,
            park_blocker: RefTo::null(),
            uncaught_exception_handler: RefTo::null(),
            thread_local_random_seed: 0,
            thread_local_random_probe: 0,
            thread_local_random_secondary_seed: 0,
        };
        let thread_ref = RefTo::new(thread);
        self.main_thread = thread_ref.erase();

        Ok(())
    }
}

#[repr(C)]
#[derive(Debug)]
struct BuiltinThread {
    object: Object,

    name: RefTo<BuiltinString>,
    priority: Int,
    daemon: Bool,
    interrupted: Bool,
    stillborn: Bool,
    eetop: Long,
    target: RefTo<Object>,
    thread_group: RefTo<BuiltinThreadGroup>,
    context_class_loader: RefTo<Object>,
    inherited_access_control_context: RefTo<Object>,
    thread_locals: RefTo<Object>,
    inheritable_thread_locals: RefTo<Object>,
    stack_size: Long,
    tid: Long,
    status: Int,
    park_blocker: RefTo<Object>,
    uncaught_exception_handler: RefTo<Object>,

    thread_local_random_seed: Int,
    thread_local_random_probe: Int,
    thread_local_random_secondary_seed: Int,
}

impl HasObjectHeader<BuiltinThread> for BuiltinThread {
    fn header(&self) -> &Object {
        &self.object
    }

    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }
}
#[repr(C)]
#[derive(Debug)]
struct BuiltinThreadGroup {
    object: Object,

    parent: RefTo<BuiltinThreadGroup>,
    name: RefTo<BuiltinString>,
    max_priority: Int,
    destroyed: Bool,
    daemon: Bool,
    n_unstarted_threads: Int,

    n_threads: Int,
    threads: RefTo<Array<RefTo<Object>>>,

    n_groups: Int,
    groups: RefTo<Array<RefTo<Object>>>,
}

impl HasObjectHeader<BuiltinThreadGroup> for BuiltinThreadGroup {
    fn header(&self) -> &Object {
        &self.object
    }

    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }
}

#[cfg(test)]
mod tests {}
