#![feature(pointer_byte_offsets)]
#![feature(offset_of)]
#![allow(clippy::new_without_default)]

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use bytecode::decode_instruction;
use bytes::BytesMut;

use parse::attributes::CodeAttribute;

use runtime::{
    error::{Frame, Throwable, ThrownState, VMError},
    object::{
        builtins::{BuiltinThread, BuiltinThreadGroup, Class, Object},
        interner::intern_string,
        mem::RefTo,
        value::RuntimeValue,
    },
    vm::VM,
};
use support::types::MethodDescriptor;
use tracing::{debug, info, trace};

pub mod bytecode;

pub struct Context {
    pub code: CodeAttribute,
    pub class: RefTo<Class>,

    pub pc: i32,
    pub is_reentry: bool,
    pub operands: Vec<RuntimeValue>,
    pub locals: Vec<RuntimeValue>,
}

impl Context {
    pub fn for_method(descriptor: &MethodDescriptor, class: RefTo<Class>) -> Self {
        let class_file = class.unwrap_ref().class_file();
        let method = class_file.methods.locate(descriptor).unwrap();

        let code = method
            .attributes
            .known_attribute::<CodeAttribute>(&class_file.constant_pool)
            .unwrap();

        Self {
            class,
            code,
            pc: 0,
            is_reentry: false,
            operands: vec![],
            locals: vec![],
        }
    }

    pub fn set_locals(&mut self, args: Vec<RuntimeValue>) {
        self.locals = args;
    }
}

pub struct BootOptions {
    pub max_stack: u64,
}

pub struct Interpreter {
    vm: VM,
    options: BootOptions,
}

impl Interpreter {
    pub fn new(vm: VM, options: BootOptions) -> Self {
        Self { vm, options }
    }
}

// Since Interpreter is a more specific version of VM, allow dereffing to vm from it
// This will allow us to call methods "through" the struct
impl Deref for Interpreter {
    type Target = VM;

    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}

impl DerefMut for Interpreter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vm
    }
}

impl Interpreter {
    pub fn run(
        &mut self,
        mut ctx: Context,
    ) -> Result<Option<RuntimeValue>, (Throwable, ThrownState)> {
        let is_overflowing_in_different_method = {
            let is_overflowing = self.vm.frames().len() > self.options.max_stack as usize;
            is_overflowing && !ctx.is_reentry
        };

        if is_overflowing_in_different_method {
            return Err(self
                .vm
                .try_make_error(VMError::StackOverflowError {})
                .map_err(|e| {
                    (
                        e,
                        ThrownState {
                            pc: ctx.pc,
                            locals: ctx.locals.clone(),
                        },
                    )
                })
                .map(|e| {
                    (
                        e,
                        ThrownState {
                            pc: ctx.pc,
                            locals: ctx.locals.clone(),
                        },
                    )
                })?);
        }

        while ctx.pc < ctx.code.code.len() as i32 {
            let slice = &ctx.code.code[ctx.pc as usize..];
            let consumed_bytes_prev = slice.len();

            let mut code_bytes = BytesMut::new();
            code_bytes.extend_from_slice(slice);

            let mut instruction_bytes = BytesMut::new();
            instruction_bytes.extend_from_slice(slice);

            let instruction =
                decode_instruction(self, &mut instruction_bytes, &ctx).map_err(|e| {
                    (
                        e,
                        ThrownState {
                            pc: ctx.pc,
                            locals: ctx.locals.clone(),
                        },
                    )
                })?;

            let consumed_bytes_post = instruction_bytes.len();
            let bytes_consumed_by_opcode = (consumed_bytes_prev - consumed_bytes_post) as i32;
            trace!(
                "opcode: {:?} consumed {} bytes",
                instruction,
                bytes_consumed_by_opcode
            );

            let progression = instruction.handle(self, &mut ctx).map_err(|e| {
                (
                    e,
                    ThrownState {
                        pc: ctx.pc,
                        locals: ctx.locals.clone(),
                    },
                )
            })?;

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
                    return Err((
                        err,
                        ThrownState {
                            pc: ctx.pc,
                            locals: ctx.locals.clone(),
                        },
                    ));
                }
            };
        }

        Ok(None)
    }

    pub fn initialise_class(&mut self, class: RefTo<Class>) -> Result<(), Throwable> {
        let class_name = class.unwrap_ref().name().clone();

        if class.unwrap_ref().is_initialised() {
            debug!(
                "Not initialising {}, class is already initialised",
                class_name
            );

            return Ok(());
        }

        let clinit = class
            .unwrap_ref()
            .class_file()
            .methods
            .locate(&("<clinit>", "()V").try_into().unwrap())
            .cloned();

        if let Some(clinit) = clinit {
            debug!("Initialising {}", class_name);

            // Need to drop our lock on the class object before running the class initialiser
            // as it could call instructions which access class data
            class.with_lock(|class| {
                class.set_initialised(true);
            });

            let code = clinit
                .attributes
                .known_attribute(&class.unwrap_ref().class_file().constant_pool)?;

            let ctx = Context {
                code,
                class,
                pc: 0,
                is_reentry: false,
                operands: vec![],
                locals: vec![],
            };

            self.vm.push_frame(Frame {
                method_name: "<clinit>".to_string(),
                class_name: class_name.clone(),
            });

            let res = self.run(ctx).map_err(|e| e.0);
            res?;

            debug!("Finished initialising {}", class_name);
            self.vm.pop_frame();
        } else {
            debug!("No clinit in {}", class_name);
            // Might as well mark this as initialised to avoid future
            // needless method lookups
            class.with_lock(|class| {
                class.set_initialised(true);
            });
        }

        Ok(())
    }

    pub fn bootstrap(&mut self) -> Result<(), Throwable> {
        // Load native modules
        use runtime::native::*;

        fn load_module(interpreter: &mut Interpreter, mut m: impl NativeModule + 'static) {
            // Setup all the methods
            m.init();

            // Load the class specified by this module
            let cls = m.get_class(&mut interpreter.vm).unwrap();

            // Just to stop us making errors with registration.
            if cls.unwrap_ref().native_module().is_some() {
                panic!("attempted to re-register module {}", m.classname());
            }

            // Apply the module to the class
            cls.with_lock(|cls| {
                cls.set_native_module(Box::new(RefCell::new(m)));
            });
        }

        load_module(self, lang::LangClass::new());
        load_module(self, lang::LangSystem::new());
        load_module(self, lang::LangObject::new());
        load_module(self, lang::LangShutdown::new());
        load_module(self, lang::LangStringUtf16::new());
        load_module(self, lang::LangRuntime::new());
        load_module(self, lang::LangDouble::new());
        load_module(self, lang::LangFloat::new());
        load_module(self, lang::LangString::new());
        load_module(self, lang::LangThrowable::new());
        load_module(self, lang::LangStackTraceElement::new());
        load_module(self, lang::LangClassLoader::new());
        load_module(self, lang::LangThread::new());

        load_module(self, jdk::JdkVM::new());
        load_module(self, jdk::JdkReflection::new());
        load_module(self, jdk::JdkCDS::new());
        load_module(self, jdk::JdkSystemPropsRaw::new());
        load_module(self, jdk::JdkUnsafe::new());
        load_module(self, jdk::JdkSignal::new());
        load_module(self, jdk::JdkScopedMemoryAccess::new());
        load_module(self, jdk::JdkBootLoader::new());

        load_module(self, io::IOFileDescriptor::new());
        load_module(self, io::IOFileOutputStream::new());
        load_module(self, io::IOUnixFileSystem::new());
        load_module(self, io::IOFileInputStream::new());

        load_module(self, security::SecurityAccessController::new());

        // Init String so that we can set the static after it's done. The clinit sets it to a default.
        let jlstr = self.class_loader().for_name("Ljava/lang/String;".into())?;
        self.initialise_class(jlstr.clone())?;

        // Load up System so that we can set up the statics
        let jlsys = self.class_loader().for_name("Ljava/lang/System;".into())?;

        {
            let statics = jlstr.unwrap_ref().statics();
            let mut statics = statics.write();
            let field = statics.get_mut("COMPACT_STRINGS").unwrap();

            field.value = Some(RuntimeValue::Integral(0_i32.into()));
        }

        {
            let statics = jlsys.unwrap_ref().statics();
            let mut statics = statics.write();
            // indicates if a security manager is possible
            // private static final int NEVER = 1;
            let field = statics
                .get_mut(&"allowSecurityManager".to_string())
                .unwrap();

            field.value = Some(RuntimeValue::Integral(1_i32.into()));
        }

        // Init thread
        let thread_class = self.class_loader().for_name("Ljava/lang/Thread;".into())?;
        self.initialise_class(thread_class.clone())?;

        let thread_group_class = self
            .class_loader()
            .for_name("Ljava/lang/ThreadGroup;".into())?;
        self.initialise_class(thread_class.clone())?;

        let thread = BuiltinThread {
            object: Object::new(
                thread_class.clone(),
                thread_class.unwrap_ref().super_class(),
            ),
            name: intern_string("main".to_string())?,
            priority: 1,
            daemon: 0,
            interrupted: 0,
            stillborn: 0,
            eetop: 0,
            target: RefTo::null(),
            thread_group: RefTo::new(BuiltinThreadGroup {
                object: Object::new(
                    thread_group_class.clone(),
                    thread_group_class.unwrap_ref().super_class(),
                ),
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
        self.set_main_thread(thread_ref.erase());

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
