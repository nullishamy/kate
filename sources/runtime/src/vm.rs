use crate::{
    error::{self, Frame, Throwable, VMError},
    object::{builtins::Object, loader::ClassLoader, mem::RefTo, value::RuntimeValue},
};

pub struct VM {
    class_loader: ClassLoader,
    frames: Vec<Frame>,

    main_thread: RefTo<Object>,
}

impl VM {
    pub fn new(class_loader: ClassLoader) -> Self {
        Self {
            class_loader,
            frames: vec![],
            main_thread: RefTo::null(),
        }
    }

    pub fn set_main_thread(&mut self, new_thread: RefTo<Object>) {
        if self.main_thread.is_not_null() {
            panic!("Cannot set main thread more than once");
        }

        self.main_thread = new_thread;
    }

    pub fn class_loader(&mut self) -> &mut ClassLoader {
        &mut self.class_loader
    }

    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }

    pub fn frames_mut(&mut self) -> &mut Vec<Frame> {
        &mut self.frames
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    /// Try and make the error. This may fail if a class fails to resolve, or object creation fails
    pub fn try_make_error(&mut self, ty: VMError) -> Result<Throwable, Throwable> {
        let cls = self
            .class_loader
            .for_name(format!("L{};", ty.class_name()).into())?;

        Ok(Throwable::Runtime(error::RuntimeException {
            message: ty.message(),
            ty: cls,
            obj: RuntimeValue::null_ref(),
            sources: self.frames.clone(),
        }))
    }

    pub fn main_thread(&self) -> RefTo<Object> {
        self.main_thread.clone()
    }
}
