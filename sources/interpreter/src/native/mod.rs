use std::collections::HashMap;

use crate::{
    error::Throwable,
    object::{
        builtins::{Class, Object},
        mem::RefTo,
        runtime::RuntimeValue,
    },
    VM,
};

pub mod io;
pub mod jdk;
pub mod lang;
pub mod security;

pub type NameAndDescriptor = (String, String);

pub type NativeStaticFunction = Box<
    dyn FnMut(
        // this-class
        RefTo<Class>,
        // args
        Vec<RuntimeValue>,
        // VM
        &mut VM,
    ) -> Result<Option<RuntimeValue>, Throwable>,
>;

pub type NativeInstanceFunction = Box<
    dyn FnMut(
        // this
        RefTo<Object>,
        // args
        Vec<RuntimeValue>,
        // VM
        &mut VM,
    ) -> Result<Option<RuntimeValue>, Throwable>,
>;

pub enum NativeFunction {
    Static(NativeStaticFunction),
    Instance(NativeInstanceFunction),
}

pub trait NativeModule {
    fn classname(&self) -> &'static str;
    fn init(&mut self);

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction>;
    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction>;

    fn get_method(&mut self, method: NameAndDescriptor) -> Option<&mut NativeFunction> {
        self.methods_mut().get_mut(&method)
    }

    fn set_method(&mut self, name: &str, descriptor: &str, func: NativeFunction) {
        self.methods_mut()
            .insert((name.to_string(), descriptor.to_string()), func);
    }

    fn get_class(&self, vm: &mut VM) -> Result<RefTo<Class>, Throwable> {
        vm.class_loader.for_name(format!("L{};", self.classname()).into())
    }
}

pub struct DefaultNativeModule {
    methods: HashMap<NameAndDescriptor, NativeFunction>,
    class_name: &'static str
}

impl NativeModule for DefaultNativeModule {
    fn classname(&self) -> &'static str {
        self.class_name
    }

    fn init(&mut self) {
        
    }

    fn methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.methods
    }
}

impl DefaultNativeModule {
    pub fn new(class_name: &'static str) -> Self {
        Self { methods: HashMap::new(), class_name }
    }
}

#[macro_export]
macro_rules! static_method {
    ($method: expr) => {
        NativeFunction::Static(Box::new($method))
    };
}

#[macro_export]
macro_rules! instance_method {
    ($method: expr) => {
        NativeFunction::Instance(Box::new($method))
    };
}

#[macro_export]
macro_rules! module_base {
    ($ty: ident) => {
        pub struct $ty {
            methods: HashMap<NameAndDescriptor, NativeFunction>,
        }

        impl $ty {
            pub fn new() -> Self {
                Self {
                    methods: HashMap::new(),
                }
            }
        }
    };
}