use std::fmt;

use enum_as_inner::EnumAsInner;
use support::descriptor::{FieldType, BaseType};

use crate::object::numeric::Floating;

use super::{builtins::Object, mem::RefTo, numeric::{Integral, IntegralType, FloatingType}};

pub enum ComputationalType {
    Category1,
    Category2,
}

#[derive(Clone, EnumAsInner)]
pub enum RuntimeValue {
    Object(RefTo<Object>),
    Integral(Integral),
    Floating(Floating),
}

impl RuntimeValue {
    pub fn null_ref() -> Self {
        RuntimeValue::Object(RefTo::null())
    }

    pub fn hash_code(&self) -> i32 {
        match self {
            RuntimeValue::Object(data) => {
                let pt = data.as_ptr();
                pt as i32
            }
            RuntimeValue::Integral(data) => data.value as i32,
            RuntimeValue::Floating(data) => data.value as i32,
        }
    }

    pub fn computational_type(&self) -> ComputationalType {
        match self {
            RuntimeValue::Integral(int) => match int.ty {
                IntegralType::Int => ComputationalType::Category1,
                IntegralType::Long => ComputationalType::Category2,
            },
            RuntimeValue::Floating(float) => match float.ty {
                FloatingType::Double => ComputationalType::Category1,
                FloatingType::Float => ComputationalType::Category2,
            },
            // These are all "reference" types
            RuntimeValue::Object(_) => ComputationalType::Category1,
        }
    }

    pub fn default_for_field(field: &FieldType) -> RuntimeValue {
        match field {
            FieldType::Base(ty) => match ty {
                BaseType::Boolean => RuntimeValue::Integral(0_i32.into()),
                BaseType::Char => RuntimeValue::Integral(0_i32.into()),
                BaseType::Float => RuntimeValue::Floating(0_f32.into()),
                BaseType::Double => RuntimeValue::Floating(0_f64.into()),
                BaseType::Byte => RuntimeValue::Integral(0_i32.into()),
                BaseType::Short => RuntimeValue::Integral(0_i32.into()),
                BaseType::Int => RuntimeValue::Integral(0_i32.into()),
                BaseType::Long => RuntimeValue::Integral(0_i64.into()),
                BaseType::Void => panic!("cannot default for void"),
            },
            FieldType::Object(_) => RuntimeValue::Object(RefTo::null()),
            FieldType::Array(_) => RuntimeValue::Object(RefTo::null()),
        }
    }
}

const UPPER_SCIENCE_BOUND: f64 = 1_000_000.0;
const LOWER_SCIENCE_BOUND: f64 = 0.000_000_1;

impl fmt::Debug for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Object(o) => {
                if o.is_null() {
                    write!(f, "null")
                } else {
                    write!(f, "{:#?}", o)
                }
            },
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
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Object(o) => {
                if o.is_null() {
                    write!(f, "null")
                } else {
                    write!(f, "[object Object]")
                }
            },
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
        }
    }
}
