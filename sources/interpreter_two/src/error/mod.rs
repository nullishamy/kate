use std::fmt;

use crate::object::WrappedClassObject;
use thiserror::Error;

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
    pub ty: WrappedClassObject,
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
