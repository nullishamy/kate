use std::{collections::HashMap, fmt, rc::Rc};

use anyhow::{Result};
use enum_as_inner::EnumAsInner;
use parking_lot::Mutex;
use parse::{
    attributes::{Attributes, KnownAttribute},
    classfile::{Addressed, ClassFile, Method, Resolvable},
    flags::ClassFileAccessFlag,
    pool::ConstantClass,
};
use support::encoding::{EncodedString, decode_string};

use crate::runtime::{
    native::NativeFunction,
    stack::{Array, ArrayPrimitive, ArrayType, RuntimeValue},
};

pub type NameAndDescriptor = (String, String);

#[derive(EnumAsInner, Clone)]
pub enum JavaObject {
    Runtime(Rc<Mutex<RuntimeObject>>),
    String(Rc<Mutex<StringObject>>),
}

impl fmt::Debug for JavaObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(_) => f.debug_tuple("-RuntimeObject-").finish(),
            // HACK: We should ***not*** be locking here, but fuck it i dont care
            // HACK2: We should also not be in a state where we can panic here, but i do not care again
            Self::String(arg0) => f.debug_tuple("String").field(&decode_string(arg0.lock().value.clone()).unwrap()).finish(),
        }
    }
}

impl PartialEq for JavaObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Runtime(l0), Self::Runtime(r0)) => Rc::ptr_eq(l0, r0),
            (Self::String(l0), Self::String(r0)) => Rc::ptr_eq(l0, r0),
            _ => false,
        }
    }
}

impl JavaObject {
    pub fn hash_code(&self) -> i64 {
        match self {
            JavaObject::Runtime(data) => Rc::as_ptr(data) as i64,
            JavaObject::String(data) => Rc::as_ptr(data) as i64,
        }
    }

    pub fn as_class_object(self) -> Rc<Mutex<ClassObject>> {
        match self {
            JavaObject::Runtime(data) => Rc::clone(&data.lock().class),
            JavaObject::String(data) => Rc::clone(&data.lock().class),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeObject {
    pub class: Rc<Mutex<ClassObject>>,
    pub is_primitive: bool,
    instance_variables: HashMap<NameAndDescriptor, RuntimeValue>,
}

impl RuntimeObject {
    pub fn new(class: Rc<Mutex<ClassObject>>) -> Self {
        Self {
            class,
            is_primitive: false,
            instance_variables: HashMap::new(),
        }
    }

    pub fn get_instance_field(&self, field: &NameAndDescriptor) -> Option<RuntimeValue> {
        self.instance_variables.get(field).cloned()
    }

    pub fn set_instance_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) {
        self.instance_variables.insert(field, value);
    }
}

#[derive(Debug, Clone)]
pub struct StringObject {
    pub class: Rc<Mutex<ClassObject>>,
    pub value: EncodedString,

    instance_variables: HashMap<NameAndDescriptor, RuntimeValue>,
}

impl StringObject {
    pub fn new(class: Rc<Mutex<ClassObject>>, value: EncodedString) -> Self {
        let mut s = Self {
            class,
            value: value.clone(),
            instance_variables: HashMap::new(),
        };

        s.instance_variables.insert(
            ("value".to_string(), "[B".to_string()),
            RuntimeValue::Array(Rc::new(Mutex::new(Array {
                ty: ArrayType::Primitive(ArrayPrimitive::Byte),
                values: value.1
                    .iter()
                    .map(|b| RuntimeValue::Integral((*b as i64).into()))
                    .collect::<Vec<RuntimeValue>>(),
            }))),
        );

        s.instance_variables.insert(
            ("coder".to_string(), "B".to_string()),
            RuntimeValue::Integral((value.0 as usize as i64).into()),
        );

        s.class.lock().set_static_field(
            ("COMPACT_STRINGS".to_string(), "Z".to_string()),
            RuntimeValue::Integral(0.into()),
        );

        s
    }

    pub fn get_instance_field(&self, field: &NameAndDescriptor) -> Option<RuntimeValue> {
        self.instance_variables.get(field).cloned()
    }

    pub fn set_instance_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) {
        self.instance_variables.insert(field, value);
    }
}

pub struct ClassObject {
    pub is_initialised: bool,

    class_file: ClassFile,

    static_fields: HashMap<NameAndDescriptor, RuntimeValue>,
    native_methods: HashMap<NameAndDescriptor, NativeFunction>,
}

impl fmt::Debug for ClassObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClassObject")
            .field("is_initialised", &self.is_initialised)
            .field("static_fields", &self.static_fields)
            .field("native_methods", &self.native_methods)
            .finish()
    }
}

impl ClassObject {
    pub fn for_runtime_object(class_file: ClassFile) -> Self {
        Self {
            class_file,
            native_methods: HashMap::new(),
            static_fields: HashMap::new(),
            is_initialised: false,
        }
    }

    pub fn is_interface(&self) -> bool {
        self.class_file
            .access_flags
            .has(ClassFileAccessFlag::INTERFACE)
    }

    pub fn get_constant<T>(&self, index: u16) -> Addressed<T> {
        self.class_file.constant_pool.address(index)
    }

    pub fn get_method(&self, name: String, descriptor: String) -> Option<Method> {
        self.class_file.methods.locate(name, descriptor).cloned()
    }

    pub fn resolve_known_attribute<T>(&self, attributes: &Attributes) -> Result<T>
    where
        T: KnownAttribute,
    {
        attributes.known_attribute(&self.class_file.constant_pool)
    }

    pub fn get_super_class(&self) -> Option<ConstantClass> {
        self.class_file.super_class.as_ref().map(|s| s.resolve())
    }

    pub fn get_class_name(&self) -> String {
        self.class_file.this_class.resolve().name.resolve().string()
    }

    pub fn set_static_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) {
        self.static_fields.insert(field, value);
    }

    pub fn get_static_field(&self, field: &NameAndDescriptor) -> Option<RuntimeValue> {
        self.static_fields.get(field).cloned()
    }

    pub fn get_native_method(&self, method: &NameAndDescriptor) -> Option<NativeFunction> {
        self.native_methods.get(method).cloned()
    }

    pub fn set_native_method(&mut self, method: NameAndDescriptor, func: NativeFunction) {
        self.native_methods.insert(method, func);
    }

    pub fn get_class_file(&self) -> &ClassFile {
        &self.class_file
    }
}
