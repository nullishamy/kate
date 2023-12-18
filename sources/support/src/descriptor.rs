use std::{iter::Peekable, str::Chars};

use anyhow::{anyhow, Result};
use enum_as_inner::EnumAsInner;

/// <BaseType> ::= 'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z'
#[derive(EnumAsInner, Debug, PartialEq, Clone, Eq, Hash)]
pub enum BaseType {
    Boolean, // Z
    Char,    // C
    Float,   // F
    Double,  // D
    Byte,    // B
    Short,   // S
    Int,     // I
    Long,    // J
    Void,    // V
}

impl ToString for BaseType {
    fn to_string(&self) -> String {
        match self {
            BaseType::Boolean => "Z",
            BaseType::Char => "C",
            BaseType::Float => "F",
            BaseType::Double => "D",
            BaseType::Byte => "B",
            BaseType::Short => "S",
            BaseType::Int => "I",
            BaseType::Long => "J",
            BaseType::Void => "V",
        }
        .to_string()
    }
}

/// <ObjectType> ::= 'L' <ClassName> ';'
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct ObjectType {
    pub class_name: String,
}

impl ToString for ObjectType {
    fn to_string(&self) -> String {
        format!("L{};", self.class_name)
    }
}

/// <ArrayType> ::= '[' <FieldType>
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct ArrayType {
    pub field_type: Box<FieldType>,
    pub dimensions: usize,
}

impl ToString for ArrayType {
    fn to_string(&self) -> String {
        format!("{}{}", "[".repeat(self.dimensions), self.field_type.to_string())
    }
}

#[derive(EnumAsInner, Debug, PartialEq, Clone, Eq, Hash)]
pub enum FieldType {
    Base(BaseType),
    Object(ObjectType),
    Array(ArrayType),
}

impl From<&'static str> for FieldType {
    fn from(value: &'static str) -> Self {
        Self::parse(value.to_string()).unwrap()
    }
}

impl From<String> for FieldType {
    fn from(value: String) -> Self {
        Self::parse(value).unwrap()
    }
}

impl FieldType {
    pub fn name(&self) -> String {
        match self {
            FieldType::Base(ty) => (match ty {
                BaseType::Boolean => "Z",
                BaseType::Char => "C",
                BaseType::Float => "F",
                BaseType::Double => "D",
                BaseType::Byte => "B",
                BaseType::Short => "S",
                BaseType::Int => "I",
                BaseType::Long => "J",
                BaseType::Void => unreachable!(),
            })
            .to_string(),
            FieldType::Object(ty) => ty.class_name.to_string(),
            FieldType::Array(ty) => ty.field_type.name(),
        }
    }
}

/// <MethodType> ::= '(' { <FieldType> } ')' <FieldType>
#[derive(Debug, PartialEq, Clone)]
pub struct MethodType {
    pub parameters: Vec<FieldType>,
    pub return_type: FieldType,
}

impl ToString for MethodType {
    fn to_string(&self) -> String {
        format!(
            "({}){}",
            self.parameters
                .iter()
                .map(|p| p.to_string())
                .collect::<String>(),
            self.return_type.to_string()
        )
    }
}

impl MethodType {
    pub fn parse(str: String) -> Result<Self> {
        let mut chars = str.chars().peekable();
        if chars.next() != Some('(') {
            return Err(anyhow!("descriptor did not start with ("));
        }

        let mut parameters = Vec::new();

        while chars.peek() != Some(&')') {
            parameters.push(FieldType::parse_from_iterator(&mut chars)?);
        }

        // Skip )
        chars.next();

        let return_type = FieldType::parse_from_iterator(&mut chars)?;

        Ok(MethodType {
            parameters,
            return_type,
        })
    }
}

impl ToString for FieldType {
    fn to_string(&self) -> String {
        match self {
            FieldType::Base(base) => base.to_string(),
            FieldType::Object(object) => object.to_string(),
            FieldType::Array(array) => array.to_string(),
        }
    }
}

impl FieldType {
    fn parse_from_iterator(chars: &mut Peekable<Chars>) -> Result<Self> {
        let first = chars.next().ok_or(anyhow!("no more chars"))?;

        Ok(match first {
            'B' => FieldType::Base(BaseType::Byte),
            'C' => FieldType::Base(BaseType::Char),
            'D' => FieldType::Base(BaseType::Double),
            'F' => FieldType::Base(BaseType::Float),
            'I' => FieldType::Base(BaseType::Int),
            'J' => FieldType::Base(BaseType::Long),
            'S' => FieldType::Base(BaseType::Short),
            'Z' => FieldType::Base(BaseType::Boolean),
            'V' => FieldType::Base(BaseType::Void),
            '[' => {
                let mut dimensions = 1;
                while let Some(ch) = chars.peek() {
                    if *ch != '[' {
                        break
                    }

                    chars.next();

                    dimensions += 1;
                }

                FieldType::Array(ArrayType {
                    dimensions,
                    field_type: Box::new(FieldType::parse_from_iterator(chars)?),
                })
            }
            'L' => FieldType::Object(ObjectType {
                class_name: chars.take_while(|c| *c != ';').collect::<String>(),
            }),
            _ => return Err(anyhow!("unknown type {first}")),
        })
    }

    pub fn parse(str: String) -> Result<Self> {
        let chars = str.chars();
        FieldType::parse_from_iterator(&mut chars.peekable())
    }
}
