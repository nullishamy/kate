use crate::{
    error::Throwable,
    VM, object::{mem::RefTo, builtins::{Class, Object}, runtime::RuntimeValue},
};

pub mod jdk;
pub mod lang;
pub mod io;

pub type NameAndDescriptor = (String, String);

pub type NativeStaticFunction = fn(
    class: RefTo<Class>,
    args: Vec<RuntimeValue>,
    vm: &mut VM,
) -> Result<Option<RuntimeValue>, Throwable>;

pub type NativeInstanceFunction = fn(
    this: RefTo<Object>,
    args: Vec<RuntimeValue>,
    vm: &mut VM,
) -> Result<Option<RuntimeValue>, Throwable>;

#[derive(Clone, Debug)]
pub enum NativeFunction {
    Static(NativeStaticFunction),
    Instance(NativeInstanceFunction),
}

pub trait NativeModule {
    fn classname() -> &'static str;

    fn methods() -> Vec<(NameAndDescriptor, NativeFunction)> {
        vec![]
    }

    fn static_fields() -> Vec<(NameAndDescriptor, RuntimeValue)> {
        vec![]
    }

    fn register(vm: &mut VM) -> Result<(), Throwable> {
        let class = vm
            .class_loader
            .for_name(Self::classname().to_string())?;

        let class = class.borrow_mut();

        for (name, method) in Self::methods() {
            class.native_methods_mut().insert(name, method);
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! static_method {
    (name: $name: expr, descriptor: $descriptor: expr => $method: expr) => {
        (
            ($name.to_string(), $descriptor.to_string()),
            NativeFunction::Static($method),
        )
    };
}

#[macro_export]
macro_rules! instance_method {
    (name: $name: expr, descriptor: $descriptor: expr => $method: expr) => {
        (
            ($name.to_string(), $descriptor.to_string()),
            NativeFunction::Instance($method),
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
