use parking_lot::Mutex;
use std::{fmt, ops, rc::Rc};

use enum_as_inner::EnumAsInner;

use crate::runtime::object::{ClassObject, JavaObject};

#[derive(Clone, Debug)]
pub enum ArrayPrimitive {
    Bool,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
}

impl ArrayPrimitive {
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            4 => Self::Bool,
            5 => Self::Char,
            6 => Self::Float,
            7 => Self::Double,
            8 => Self::Byte,
            9 => Self::Short,
            10 => Self::Int,
            11 => Self::Long,
            _ => todo!("unknown array type {}", tag),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArrayType {
    Object(Rc<Mutex<ClassObject>>),
    Primitive(ArrayPrimitive),
}

#[derive(Debug)]
pub struct Array {
    pub ty: ArrayType,
    pub values: Vec<RuntimeValue>,
}

#[derive(Clone, EnumAsInner)]
pub enum RuntimeValue {
    Integral(Integral),
    Floating(Floating),
    Object(JavaObject),
    Array(Rc<Mutex<Array>>),
    Null,
}

impl RuntimeValue {
    pub fn category_type(&self) -> u8 {
        match self {
            RuntimeValue::Integral(data) => match data.ty {
                IntegralType::Int => 1,
                IntegralType::Long => 2,
            },
            RuntimeValue::Floating(data) => match data.ty {
                FloatingType::Double => 2,
                FloatingType::Float => 1,
            },
            // These are all technically 'reference' types
            RuntimeValue::Object(_) => 1,
            RuntimeValue::Array(_) => 1,
            RuntimeValue::Null => 1,
        }
    }
}

#[derive(Clone, EnumAsInner, Debug, Copy, PartialEq)]
pub enum IntegralType {
    Int,
    Long,
}

#[derive(Clone, Debug, Copy)]
pub struct Integral {
    pub ty: IntegralType,
    pub value: i64,
}

impl Integral {
    pub fn int(value: i64) -> Self {
        Self {
            ty: IntegralType::Int,
            value,
        }
    }

    pub fn long(value: i64) -> Self {
        Self {
            ty: IntegralType::Long,
            value,
        }
    }
}

impl ops::Add for Integral {
    type Output = i64;

    fn add(self, rhs: Self) -> Self::Output {
        self.value + rhs.value
    }
}

impl ops::BitOr for Integral {
    type Output = i64;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.value | rhs.value
    }
}

impl ops::BitAnd for Integral {
    type Output = i64;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.value & rhs.value
    }
}

impl ops::Sub for Integral {
    type Output = i64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.value - rhs.value
    }
}

impl ops::Mul for Integral {
    type Output = i64;

    fn mul(self, rhs: Self) -> Self::Output {
        self.value * rhs.value
    }
}

impl ops::Div for Integral {
    type Output = i64;

    fn div(self, rhs: Self) -> Self::Output {
        self.value / rhs.value
    }
}

impl PartialEq for Integral {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl PartialEq for Floating {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl PartialOrd for Integral {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl PartialOrd for Floating {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl ops::Deref for Integral {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl From<i64> for Integral {
    fn from(val: i64) -> Integral {
        Integral::int(val)
    }
}

#[derive(Clone, EnumAsInner, Debug, Copy, PartialEq)]
pub enum FloatingType {
    Double,
    Float,
}
#[derive(Clone, Debug, Copy)]
pub struct Floating {
    pub ty: FloatingType,
    pub value: f64,
}

impl Floating {
    pub fn double(value: f64) -> Self {
        Self {
            ty: FloatingType::Double,
            value,
        }
    }

    pub fn float(value: f64) -> Self {
        Self {
            ty: FloatingType::Float,
            value,
        }
    }
}

impl ops::Add for Floating {
    type Output = f64;

    fn add(self, rhs: Self) -> Self::Output {
        self.value + rhs.value
    }
}

impl ops::Sub for Floating {
    type Output = f64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.value - rhs.value
    }
}

impl ops::Mul for Floating {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        self.value * rhs.value
    }
}

impl ops::Div for Floating {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.value / rhs.value
    }
}

impl ops::Deref for Floating {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl From<f64> for Floating {
    fn from(val: f64) -> Floating {
        Floating::float(val)
    }
}

impl fmt::Debug for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integral(arg0) => f.debug_tuple("Integral").field(arg0).finish(),
            Self::Floating(arg0) => f.debug_tuple("Floating").field(arg0).finish(),
            Self::Object(arg0) => f.debug_tuple("Object").field(arg0).finish(),
            Self::Array(arg0) => f.debug_tuple("Array").field(arg0).finish(),
            Self::Null => write!(f, "Null"),
        }
    }
}
