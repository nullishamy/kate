//! provides various types for use in the JVM.
//! mainly wraps primitives to make a more usable API out of them.

use std::sync::Arc;

use crate::runtime::heap::object::JvmObject;

use crate::structs::descriptor::DescriptorReferenceType;
use crate::structs::JvmPointer;
use crate::DescriptorType;
use enum_as_inner::EnumAsInner;

/// The bool JVM type
pub type Boolean = bool;

/// The byte JVM type
pub type Byte = i8;

/// The short JVM type
pub type Short = i16;

/// The int JVM type
pub type Int = i32;

/// The long JVM type
pub type Long = i64;

/// The char JVM type
pub type Char = char;

/// The float JVM type
pub type Float = f32;

/// The double JVM type
pub type Double = f64;

/// The return address JVM type
pub type ReturnAddress = JvmPointer;

/// refers to a primitive with a value attached to it
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

/// refers to just a primitive type, with no value
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

/// refers to a reference type, or a primitive type.
/// used for fields and method parameters
#[derive(Clone, Debug, EnumAsInner)]
pub enum RefOrPrim {
    Reference(ReferenceType),
    Primitive(PrimitiveWithValue),
}

impl RefOrPrim {
    /// get the underlying descriptor type of a RefOrPrim
    /// this returns None if there is no type associated with this value (`null`)
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
