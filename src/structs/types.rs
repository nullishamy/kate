use std::sync::Arc;

use crate::runtime::heap::object::JVMObject;

use crate::structs::JVMPointer;
use enum_as_inner::EnumAsInner;

pub type Boolean = bool;
pub type Byte = i8;
pub type Short = i16;
pub type Int = i32;
pub type Long = i64;
pub type Char = char;
pub type Float = f32;
pub type Double = f64;
pub type ReturnAddress = JVMPointer;

#[derive(PartialEq, Clone, Debug, EnumAsInner)]
pub enum PrimitiveWithValue {
    Boolean(Boolean),
    Byte(Byte),
    Short(Short),
    Int(Int),
    Long(Long),
    Char(Char),
    Float(Float),
    Double(Double),
}

#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveType {
    Boolean,
    Byte,
    Short,
    Int,
    Long,
    Char,
    Float,
    Double,
}

#[derive(Clone, Debug)]
pub enum ReferenceType {
    Class(Arc<JVMObject>),
    Null,
}

#[derive(Clone, Debug)]
pub enum RefOrPrim {
    Reference(ReferenceType),
    Primitive(PrimitiveWithValue),
}
