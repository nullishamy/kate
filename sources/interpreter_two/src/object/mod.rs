use std::{collections::HashMap, fmt, rc::Rc};

use enum_as_inner::EnumAsInner;
use parking_lot::RwLock;
use parse::{classfile::{Method, Methods, Addressed}, pool::ConstantPool};

use crate::native::{NativeFunction, NameAndDescriptor};

use self::numeric::{Integral, Floating};

pub mod array;
pub mod classloader;
pub mod numeric;

/// Any Java Object. We implement a ClassObject type which represents the java/lang/Class of an object.
/// There's some weird structuring going on here because all objects have a Class object, but Class objects are also objects.
pub trait Object: fmt::Debug {
    fn class(&self) -> Option<WrappedClassObject>;
}

#[derive(Debug)]
pub struct ClassObject {
    // java/lang/Class, because all "class objects" are based on this class
    pub meta_class_object: Option<WrappedClassObject>,
    native_methods: HashMap<NameAndDescriptor, NativeFunction>,
    methods: Methods,
    pool: ConstantPool,
    is_initialised: bool,
    class_name: String
}

impl Object for ClassObject {
    fn class(&self) -> Option<WrappedClassObject> {
        self.meta_class_object.clone()
    }
}

impl ClassObject {
    pub fn new(meta: Option<WrappedClassObject>, methods: Methods, pool: ConstantPool, name: String) -> Self {
        Self {
            meta_class_object: meta,
            native_methods: HashMap::new(),
            methods,
            pool,
            is_initialised: false,
            class_name: name
        }
    }

    pub fn get_class_name(&self) -> &String {
        &self.class_name
    }

    pub fn is_initialised(&self) -> bool {
        self.is_initialised
    }

    pub fn set_initialised(&mut self, val: bool) {
        self.is_initialised = val;
    }
    
    pub fn register_native(&mut self,name: NameAndDescriptor, method: NativeFunction) {
        self.native_methods.insert(name, method);
    }

    pub fn fetch_native(&self, name: NameAndDescriptor) -> Option<NativeFunction> {
        self.native_methods.get(&name).cloned()
    }

    pub fn get_method(&self, name: NameAndDescriptor) -> Option<Method> {
        self.methods.locate(name.0, name.1).cloned()
    }

    pub fn constant_pool(&self) -> &ConstantPool {
        &self.pool
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum RuntimeValue {
    Object(WrappedObject),
    Array(array::Array),
    Integral(Integral),
    Floating(Floating),
    Null
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Object(_) => write!(f, "[object Object]"),
            RuntimeValue::Array(data) => write!(f, "[{}]", data.values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            RuntimeValue::Integral(data) => write!(f, "{}", data.value),
            RuntimeValue::Floating(data) => write!(f, "{}", data.value),
            RuntimeValue::Null => write!(f, "null"),
        }
    }
}

pub type WrappedClassObject = Rc<RwLock<ClassObject>>;
pub type WrappedObject = Rc<RwLock<dyn Object>>;
