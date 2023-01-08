use crate::runtime::{native::NativeFunction, object::NameAndDescriptor, stack::RuntimeValue};
use anyhow::Result;

use crate::interpreter::Interpreter;

pub mod jdk;
pub mod lang;

pub trait NativeModule {
    fn classname() -> &'static str;

    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![]
    }
    fn static_fields() -> Vec<(NameAndDescriptor, RuntimeValue)> {
        vec![]
    }

    fn register(interpreter: &Interpreter) -> Result<()> {
        let class = interpreter.load_class(Self::classname().to_string())?;
        let mut class = class.lock();

        for (name, method) in Self::methods() {
            class.set_native_method(name, method);
        }

        for (name, value) in Self::static_fields() {
            class.set_static_field(name, value);
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! static_method {
    (name: $name: expr, descriptor: $descriptor: expr => $method: expr) => {
        (
            ($name.to_string(), $descriptor.to_string()),
            $crate::runtime::native::NativeFunction::Static($method),
        )
    };
}

#[macro_export]
macro_rules! instance_method {
    (name: $name: expr, descriptor: $descriptor: expr => $method: expr) => {
        (
            ($name.to_string(), $descriptor.to_string()),
            $crate::runtime::native::NativeFunction::Instance($method),
        )
    };
}

#[macro_export]
macro_rules! field {
    (name: $name: expr, descriptor: $descriptor: expr => $value: expr) => {
        (
            $crate::runtime::object::NameAndDescriptor {
                name: $name.to_string(),
                descriptor: $descriptor.to_string(),
            },
            $value,
        )
    };
}

#[macro_export]
macro_rules! class {
    ($name: ident) => {
        pub struct $name;
    };
}
