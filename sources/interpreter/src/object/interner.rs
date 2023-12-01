use std::{collections::HashMap, sync::OnceLock};

use parking_lot::Mutex;
use send_wrapper::SendWrapper;
use support::encoding::encode_string;

use crate::error::Throwable;

use super::{
    builtins::{Array, ArrayPrimitive, ArrayType, BuiltinString, Class, Object},
    mem::RefTo,
};

lazy_static::lazy_static! {
    static ref INTERNER: SendWrapper<Mutex<OnceLock<StringInterner>>> = {
        SendWrapper::new(Mutex::new(OnceLock::new()))
    };
}

pub fn set_interner(interner: StringInterner) {
    let _interner = &INTERNER;
    let mut _interner = _interner.lock();
    _interner.set(interner).unwrap();
}

pub fn intern_string(value: std::string::String) -> Result<RefTo<BuiltinString>, Throwable> {
    let _interner = &INTERNER;
    let mut _interner = _interner.lock();
    let interner = _interner.get_mut().unwrap();

    interner.intern(value)
}

pub fn interner_meta_class() -> RefTo<Class> {
    let _interner = &INTERNER;
    let mut _interner = _interner.lock();
    let interner = _interner.get().unwrap();
    interner.meta_class()
}

#[derive(Debug)]
pub struct StringInterner {
    string_class: RefTo<Class>,
    super_class: RefTo<Class>,

    strings: HashMap<String, RefTo<BuiltinString>>,
}

impl StringInterner {
    pub fn new(string_class: RefTo<Class>, super_class: RefTo<Class>) -> Self {
        Self {
            string_class,
            super_class,
            strings: HashMap::new(),
        }
    }

    pub fn intern(&mut self, value: String) -> Result<RefTo<BuiltinString>, Throwable> {
        if let Some(res) = self.strings.get(&value) {
            return Ok(res.clone());
        }

        let (encoding, bytes) = encode_string(value.clone())?;
        let array = Array::<u8>::from_vec(
            ArrayType::Primitive(ArrayPrimitive::Byte),
            "[B".to_string(),
            bytes,
        );

        let obj = BuiltinString {
            object: Object::new(self.string_class.clone(), self.super_class.clone()),
            value: array,
            coder: encoding.coder(),
            hash: 0,
            hash_is_zero: 1,
        };

        let str = RefTo::new(obj);
        self.strings.insert(value, str.clone());

        Ok(str)
    }

    fn meta_class(&self) -> RefTo<Class> {
        self.string_class.clone()
    }
}
