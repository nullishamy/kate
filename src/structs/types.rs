use std::sync::Arc;

use crate::runtime::heap::object::JvmObject;

use crate::structs::descriptor::DescriptorReferenceType;
use crate::structs::JvmPointer;
use crate::DescriptorType;
use enum_as_inner::EnumAsInner;

pub type Boolean = bool;
pub type Byte = i8;
pub type Short = i16;
pub type Int = i32;
pub type Long = i64;
pub type Char = char;
pub type Float = f32;
pub type Double = f64;
pub type ReturnAddress = JvmPointer;

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

#[derive(Clone, Debug, EnumAsInner)]
pub enum ReferenceType {
    Class(Arc<JvmObject>),
    Null,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum RefOrPrim {
    Reference(ReferenceType),
    Primitive(PrimitiveWithValue),
}

impl RefOrPrim {
    pub fn get_type(&self) -> Option<DescriptorType> {
        match self {
            RefOrPrim::Reference(d) => match d {
                ReferenceType::Class(d) => {
                    Some(DescriptorType::Reference(DescriptorReferenceType {
                        internal_name: d.class.this_class.name.str.clone(),
                    }))
                }
                ReferenceType::Null => None,
            },
            RefOrPrim::Primitive(p) => Some(DescriptorType::Primitive(match p {
                PrimitiveWithValue::Boolean(_) => PrimitiveType::Boolean,
                PrimitiveWithValue::Byte(_) => PrimitiveType::Byte,
                PrimitiveWithValue::Short(_) => PrimitiveType::Short,
                PrimitiveWithValue::Int(_) => PrimitiveType::Int,
                PrimitiveWithValue::Long(_) => PrimitiveType::Long,
                PrimitiveWithValue::Char(_) => PrimitiveType::Char,
                PrimitiveWithValue::Float(_) => PrimitiveType::Float,
                PrimitiveWithValue::Double(_) => PrimitiveType::Double,
            })),
        }
    }
}
