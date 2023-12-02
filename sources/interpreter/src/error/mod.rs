use std::fmt;

use thiserror::Error;

use crate::object::{mem::RefTo, builtins::Class, runtime::RuntimeValue};

pub enum VMError {
    ArrayIndexOutOfBounds {
        at: i64
    },
    NullPointerException
}

impl VMError {
    pub fn class_name(&self) -> &'static str {
        match self {
            VMError::ArrayIndexOutOfBounds { .. } => "java/lang/ArrayIndexOutOfBoundsException",
            VMError::NullPointerException => "java/lang/NullPointerException",
        }
    }

    pub fn message(&self) -> String {
        let ctx = match self {
            VMError::ArrayIndexOutOfBounds { at } => format!("OOB @ {}", at),
            VMError::NullPointerException => format!("NPE"),
        };

        format!("{}: {}", self.class_name(), ctx)
    }
}

#[derive(Error, Debug)]
pub enum Throwable {
    #[error(transparent)]
    Runtime(RuntimeException),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[macro_export]
macro_rules! internal {
    ($msg:literal $(,)?) => {
        $crate::Throwable::Internal(anyhow::anyhow!($msg))
    };
    ($err:expr $(,)?) => {
        $crate::Throwable::Internal(anyhow::anyhow!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Throwable::Internal(anyhow::anyhow!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! internalise {
    () => {
        |f| $crate::internal!(f)
    };
}
#[derive(Error, Debug, Clone)]
#[error("at {class_name}.{method_name}")]
pub struct Frame {
    pub method_name: String,
    pub class_name: String,
}

#[derive(Error)]
#[error("{message}")]
pub struct RuntimeException {
    pub message: String,
    pub ty: RefTo<Class>,
    pub obj: RuntimeValue,
    pub sources: Vec<Frame>,
}

impl fmt::Debug for RuntimeException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeException")
            .field("message", &self.message)
            .field("ty", &&"<ty>")
            .field("sources", &self.sources)
            .finish()
    }
}
