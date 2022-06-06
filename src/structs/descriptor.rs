use std::iter::Peekable;
use std::str::Chars;

use anyhow::{anyhow, Result};
use enum_as_inner::EnumAsInner;
use tracing::debug;

use crate::structs::types::PrimitiveType;

pub mod notation {
    // i8
    pub const BYTE: char = 'B';

    // UTF-16 Char
    pub const CHAR: char = 'C';

    // f64
    pub const DOUBLE: char = 'D';

    // f32
    pub const FLOAT: char = 'F';

    // u32
    pub const INT: char = 'I';

    // u64
    pub const LONG: char = 'J';

    // Class type, described by an 'internal class name'
    // preceding the token
    pub const CLASS: char = 'L';

    // i16
    pub const SHORT: char = 'S';

    // bool
    pub const BOOLEAN: char = 'Z';

    // void
    pub const VOID: char = 'V';

    // Array type, who's reference type is
    // described by an 'internal class name'
    // preceding the token. Ends with a ';'
    pub const ARRAY: char = '[';

    pub const END_REFERENCE: char = ';';
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    raw: String,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().peekable(),
            raw: src.to_string(),
        }
    }

    fn parse_method_descriptor(mut self) -> Result<MethodDescriptor> {
        self.expect_next('(')?;

        let mut parameters: Vec<DescriptorType> = Vec::new();

        while self.peek_next()? != ')' {
            parameters.push(self.parse_descriptor_type()?);
        }

        self.expect_next(')')?;

        let return_type = self.parse_descriptor_type()?;

        Ok(MethodDescriptor {
            parameters,
            return_type,
            raw: self.raw,
        })
    }

    fn parse_array_type(&mut self) -> Result<DescriptorArrayType> {
        let mut count = 0;
        while self.peek_next()? == notation::ARRAY {
            count += 1;
            self.consume_next()?;
        }

        let type_ = self.parse_descriptor_type()?;

        Ok(DescriptorArrayType {
            type_: Box::new(type_),
            dimensions: count,
        })
    }

    fn parse_reference_type(&mut self) -> Result<DescriptorReferenceType> {
        let mut out = String::new();

        while self.peek_next()? != notation::END_REFERENCE {
            out.push(self.consume_next()?);
        }
        self.consume_next()?; // consume the end reference
        Ok(DescriptorReferenceType { internal_name: out })
    }

    fn parse_descriptor_type(&mut self) -> Result<DescriptorType> {
        let next = self.consume_next()?;

        return match next {
            notation::BYTE => Ok(DescriptorType::Primitive(PrimitiveType::Byte)),
            notation::CHAR => Ok(DescriptorType::Primitive(PrimitiveType::Char)),
            notation::DOUBLE => Ok(DescriptorType::Primitive(PrimitiveType::Double)),
            notation::FLOAT => Ok(DescriptorType::Primitive(PrimitiveType::Float)),
            notation::INT => Ok(DescriptorType::Primitive(PrimitiveType::Int)),
            notation::SHORT => Ok(DescriptorType::Primitive(PrimitiveType::Short)),
            notation::BOOLEAN => Ok(DescriptorType::Primitive(PrimitiveType::Boolean)),
            notation::LONG => Ok(DescriptorType::Primitive(PrimitiveType::Long)),
            notation::VOID => Ok(DescriptorType::Void),

            notation::CLASS => Ok(DescriptorType::Reference(self.parse_reference_type()?)),
            notation::ARRAY => Ok(DescriptorType::Array(self.parse_array_type()?)),

            _ => Err(anyhow!("unknown type identifier {}", next)),
        };
    }

    fn consume_next(&mut self) -> Result<char> {
        self.chars.next().ok_or_else(|| anyhow!("out of chars"))
    }

    fn peek_next(&mut self) -> Result<char> {
        self.chars
            .peek()
            .copied()
            .ok_or_else(|| anyhow!("out of chars"))
    }

    fn expect_next(&mut self, expect: char) -> Result<()> {
        let next = self.consume_next()?;

        if next != expect {
            return Err(anyhow!("expected {} got {}", expect, next));
        }

        Ok(())
    }
}

/*
    Examples:
    (Ljava/lang/String;)V - 1 parameter, a reference type of java/lang/String
    with a return type of void

    (IDLjava/lang/Thread;)Ljava/lang/Object; - 3 parameters, int, double and reference type java/lang/Thread
    with a return type of reference type java/lang/Object
*/

#[derive(Clone, Debug)]
pub struct MethodDescriptor {
    pub parameters: Vec<DescriptorType>,
    pub return_type: DescriptorType,

    raw: String,
}

impl MethodDescriptor {
    pub fn parse(descriptor: &str) -> Result<Self> {
        debug!("parsing method descriptor {}", descriptor);

        let parser = Parser::new(descriptor);

        parser.parse_method_descriptor()
    }
}

#[derive(Clone, Debug)]
pub struct FieldDescriptor {
    _type: DescriptorType,
    raw: String,
}

impl FieldDescriptor {
    pub fn parse(descriptor: &str) -> Result<Self> {
        debug!("parsing field descriptor {}", descriptor);
        let mut parser = Parser::new(descriptor);

        let type_ = parser.parse_descriptor_type()?;

        Ok(Self {
            _type: type_,
            raw: parser.raw,
        })
    }
}

#[derive(PartialEq, Debug, Clone, EnumAsInner)]
pub enum DescriptorType {
    Reference(DescriptorReferenceType),
    Primitive(PrimitiveType),
    Array(DescriptorArrayType),
    Void,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DescriptorArrayType {
    pub type_: Box<DescriptorType>,
    pub dimensions: u16,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DescriptorReferenceType {
    pub internal_name: String,
}

#[derive(Clone, Debug)]
pub enum Descriptor {
    Field(FieldDescriptor),
    Method(MethodDescriptor),
}

impl ToString for MethodDescriptor {
    fn to_string(&self) -> String {
        self.raw.clone()
    }
}

pub fn test_descriptor_parsing() {
    /*
        one_array_param:([Ljava/lang/String;)V
        two_array_params:([Ljava/lang/String;[Ljava/lang/String;)V
        one_two_darray_param:([[Ljava/lang/String;)V
        one_three_darray_param:([[[Ljava/lang/String;)V
        one_ref_param:(Ljava/lang/String;)V
        two_ref_params:(Ljava/lang/String;Ljava/lang/String;)V
    */

    let one_array_param = "([Ljava/lang/String;)V";
    let two_array_params = "([Ljava/lang/String;[Ljava/lang/String;)V";
    let one_two_darray_param = "([[Ljava/lang/String;)V";
    let one_three_darray_param = "([[[Ljava/lang/String;)V";
    let one_ref_param = "(Ljava/lang/String;)V";
    let two_ref_params = "(Ljava/lang/String;Ljava/lang/String;)V";

    let _all_valid = [
        MethodDescriptor::parse(one_array_param).unwrap(),
        MethodDescriptor::parse(two_array_params).unwrap(),
        MethodDescriptor::parse(one_two_darray_param).unwrap(),
        MethodDescriptor::parse(one_three_darray_param).unwrap(),
        MethodDescriptor::parse(one_ref_param).unwrap(),
        MethodDescriptor::parse(two_ref_params).unwrap(),
    ];
}
