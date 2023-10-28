use super::{RuntimeValue, WrappedObject, WrappedClassObject};


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

#[derive(Debug, Clone)]
pub enum ArrayType {
    Object(WrappedClassObject),
    Primitive(ArrayPrimitive),
}

#[derive(Debug, Clone)]
pub struct Array {
    pub ty: ArrayType,
    pub values: Vec<RuntimeValue>
}

impl Array {
    pub fn len(&self) -> usize {
        self.values.len()
    }
}