use std::fmt;

use crate::descriptor::{FieldType, MethodType};

pub type MethodName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodDescriptor(String, MethodType);

impl MethodDescriptor {
    pub fn new(name: String, ty: MethodType) -> Self {
        Self(name, ty)
    }

    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn descriptor(&self) -> &MethodType {
        &self.1
    }
}

impl<T, U> TryFrom<(T, U)> for MethodDescriptor
where
    T: Into<String>,
    U: Into<String>,
{
    type Error = anyhow::Error;

    fn try_from(value: (T, U)) -> Result<Self, Self::Error> {
        Ok(Self(value.0.into(), MethodType::parse(value.1.into())?))
    }
}

pub type FieldName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldDescriptor(String, FieldType);

impl FieldDescriptor {
    pub fn new(name: String, ty: FieldType) -> Self {
        Self(name, ty)
    }

    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn descriptor(&self) -> &FieldType {
        &self.1
    }
}

impl<T, U> TryFrom<(T, U)> for FieldDescriptor
where
    T: Into<String>,
    U: Into<String>,
{
    type Error = anyhow::Error;

    fn try_from(value: (T, U)) -> Result<Self, Self::Error> {
        Ok(Self(value.0.into(), FieldType::parse(value.1.into())?))
    }
}

impl fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.1.to_string())
    }
}

impl fmt::Display for FieldDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.1.to_string())
    }
}
