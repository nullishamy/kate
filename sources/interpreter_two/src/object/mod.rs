use anyhow::{Context, Result};
use std::{collections::HashMap, fmt, rc::Rc};

use enum_as_inner::EnumAsInner;
use parking_lot::RwLock;
use parse::{
    classfile::{Method, Methods},
    flags::{ClassFileAccessFlag, ClassFileAccessFlags},
    pool::ConstantPool,
};
use support::encoding::encode_string;

use crate::native::{NameAndDescriptor, NativeFunction};

use self::{
    array::Array,
    numeric::{Floating, Integral},
};

pub mod array;
pub mod classloader;
pub mod numeric;
pub mod statics;
pub mod string;

/// Any Java Object. We implement a ClassObject type which represents the java/lang/Class of an object.
/// There's some weird structuring going on here because all objects have a Class object, but Class objects are also objects.
pub trait Object: fmt::Debug {
    fn class(&self) -> Option<WrappedClassObject>;
    fn super_class(&self) -> Option<WrappedClassObject>;

    fn get_instance_field(&self, field: NameAndDescriptor) -> Result<RuntimeValue>;
    fn set_instance_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) -> Result<()>;

    fn access_flags(&self) -> ClassFileAccessFlags;
    fn is_interface(&self) -> bool {
        self.access_flags().has(ClassFileAccessFlag::INTERFACE)
    }
}

#[derive(Debug)]
pub struct RuntimeObject {
    pub class_object: WrappedClassObject,
    instance_fields: HashMap<NameAndDescriptor, RuntimeValue>,
}

impl RuntimeObject {
    pub fn new(class_object: WrappedClassObject) -> Self {
        Self {
            class_object,
            instance_fields: HashMap::new(),
        }
    }
}

impl Object for RuntimeObject {
    fn class(&self) -> Option<WrappedClassObject> {
        Some(Rc::clone(&self.class_object))
    }

    fn super_class(&self) -> Option<WrappedClassObject> {
        self.class_object.read().super_class()
    }

    fn get_instance_field(&self, field: NameAndDescriptor) -> Result<RuntimeValue> {
        self.instance_fields
            .get(&field)
            .cloned()
            .context(format!("no field {:#?}", field))
    }

    fn set_instance_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) -> Result<()> {
        self.instance_fields.insert(field, value);

        Ok(())
    }

    fn access_flags(&self) -> ClassFileAccessFlags {
        self.class_object.read().flags
    }
}

#[derive(Debug)]
pub struct ClassObject {
    // java/lang/Class, because all "class objects" are based on this class
    pub meta_class_object: Option<WrappedClassObject>,
    super_class: Option<WrappedClassObject>,

    native_methods: HashMap<NameAndDescriptor, NativeFunction>,
    instance_fields: HashMap<NameAndDescriptor, RuntimeValue>,
    flags: ClassFileAccessFlags,
    methods: Methods,
    pool: ConstantPool,
    is_initialised: bool,
    class_name: String,
}

impl Object for ClassObject {
    fn class(&self) -> Option<WrappedClassObject> {
        self.meta_class_object.clone()
    }

    fn super_class(&self) -> Option<WrappedClassObject> {
        self.super_class.clone()
    }

    fn access_flags(&self) -> ClassFileAccessFlags {
        self.flags
    }

    fn get_instance_field(&self, field: NameAndDescriptor) -> Result<RuntimeValue> {
        self.instance_fields
            .get(&field)
            .cloned()
            .context(format!("no field {:#?}", field))
    }

    fn set_instance_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) -> Result<()> {
        self.instance_fields.insert(field, value);

        Ok(())
    }
}

impl ClassObject {
    pub fn new(
        meta: Option<WrappedClassObject>,
        super_class: Option<WrappedClassObject>,
        methods: Methods,
        pool: ConstantPool,
        flags: ClassFileAccessFlags,
        name: String,
    ) -> Self {
        Self {
            meta_class_object: meta,
            super_class,
            native_methods: HashMap::new(),
            instance_fields: HashMap::new(),
            methods,
            flags,
            pool,
            is_initialised: false,
            class_name: name,
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

    pub fn register_native(&mut self, name: NameAndDescriptor, method: NativeFunction) {
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

#[derive(Debug)]
pub struct StringObject {
    // java/lang/String,
    pub string_class: WrappedClassObject,
    native_methods: HashMap<NameAndDescriptor, NativeFunction>,
    instance_fields: HashMap<NameAndDescriptor, RuntimeValue>,
}

impl Object for StringObject {
    fn class(&self) -> Option<WrappedClassObject> {
        Some(Rc::clone(&self.string_class))
    }

    fn super_class(&self) -> Option<WrappedClassObject> {
        self.string_class.read().super_class()
    }

    fn get_instance_field(&self, field: NameAndDescriptor) -> Result<RuntimeValue> {
        self.instance_fields
            .get(&field)
            .cloned()
            .context(format!("no field {:#?}", field))
    }

    fn set_instance_field(&mut self, field: NameAndDescriptor, value: RuntimeValue) -> Result<()> {
        self.instance_fields.insert(field, value);

        Ok(())
    }

    fn access_flags(&self) -> ClassFileAccessFlags {
        self.string_class.read().flags
    }
}

impl StringObject {
    pub fn new(java_lang_string: WrappedClassObject, value: String) -> Result<Self> {
        let mut s = Self {
            string_class: java_lang_string,
            native_methods: HashMap::new(),
            instance_fields: HashMap::new(),
        };

        let (_method, bytes) = encode_string(value)?;
        let arr = RuntimeValue::Array(Rc::new(RwLock::new(Array {
            ty: array::ArrayType::Primitive(array::ArrayPrimitive::Byte),
            values: bytes
                .iter()
                .map(|b| RuntimeValue::Integral((*b as i8).into()))
                .collect(),
        })));

        s.set_instance_field(("value".to_string(), "[B".to_string()), arr)?;
        s.set_instance_field(
            ("coder".to_string(), "B".to_string()),
            RuntimeValue::Integral((1_i32).into()),
        )?;

        Ok(s)
    }

    pub fn register_native(&mut self, name: NameAndDescriptor, method: NativeFunction) {
        self.native_methods.insert(name, method);
    }

    pub fn fetch_native(&self, name: NameAndDescriptor) -> Option<NativeFunction> {
        self.native_methods.get(&name).cloned()
    }

    pub fn get_method(&self, name: NameAndDescriptor) -> Option<Method> {
        self.string_class.read().get_method(name)
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum RuntimeValue {
    Object(WrappedObject),
    Array(WrappedArray),
    Integral(Integral),
    Floating(Floating),
    Null,
}

impl RuntimeValue {
    pub fn hash_code(&self) -> i32 {
        match self {
            RuntimeValue::Object(data) => {
                let pt = Rc::as_ptr(data);
                pt as *const () as i32
            },
            RuntimeValue::Array(data) => {
                let pt = Rc::as_ptr(data);
                pt as *const () as i32
            },
            RuntimeValue::Integral(data) => data.value as i32,
            RuntimeValue::Floating(data) => data.value as i32,
            RuntimeValue::Null => 0,
        }
    }
}

const UPPER_SCIENCE_BOUND: f64 = 1_000_000.0;
const LOWER_SCIENCE_BOUND: f64 = 0.000_000_1;

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Object(_) => write!(f, "[object Object]"),
            RuntimeValue::Array(data) => write!(
                f,
                "[{}]",
                data.read()
                    .values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RuntimeValue::Integral(data) => write!(f, "{}", data.value),
            RuntimeValue::Floating(data) => {
                // Just our custom implementation of floats, so we get reasonable output
                if data.value > UPPER_SCIENCE_BOUND {
                    write!(f, "{:+e}", data.value)
                } else if data.value < LOWER_SCIENCE_BOUND {
                    write!(f, "{:-e}", data.value)
                } else {
                    write!(f, "{:.3}", data.value)
                }
            }
            RuntimeValue::Null => write!(f, "null"),
        }
    }
}

pub type WrappedClassObject = Rc<RwLock<ClassObject>>;
pub type WrappedObject = Rc<RwLock<dyn Object>>;
pub type WrappedArray = Rc<RwLock<Array>>;
