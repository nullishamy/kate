use anyhow::Result;

use crate::{object::{RuntimeValue, WrappedClassObject, WrappedObject}, VM};

pub type NameAndDescriptor = (String, String);

pub type NativeStaticFunction = fn(
    class: WrappedClassObject,
    args: Vec<RuntimeValue>,
    vm: &mut VM,
) -> Result<Option<RuntimeValue>>;

pub type NativeInstanceFunction = fn(
    this: WrappedObject,
    args: Vec<RuntimeValue>,
    vm: &mut VM,
) -> Result<Option<RuntimeValue>>;

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

    fn register(vm: &mut VM) -> Result<()> {
        let class = vm.class_loader.load_class(Self::classname().to_string())?;
        let mut class = class.write();
        for (name, method) in Self::methods() {
            class.register_native(name, method);
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

#[macro_export]
macro_rules! class {
    ($name: ident) => {
        pub struct $name;
    };
}
