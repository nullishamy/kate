use std::iter::Peekable;
use std::str::Chars;

use anyhow::{anyhow, Result};

use crate::structs::primitive::PrimitiveType;

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
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().peekable(),
        }
    }

    fn parse_method_descriptor(&mut self) -> Result<MethodDescriptor> {
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
        })
    }

    fn parse_array_type(&mut self) -> Result<ArrayType> {
        let mut count = 0;
        while self.peek_next()? == notation::ARRAY {
            count += 1;
            self.consume_next()?;
        }

        let _type = self.parse_descriptor_type()?;

        Ok(ArrayType {
            _type: Box::new(_type),
            dimensions: count,
        })
    }

    fn parse_reference_type(&mut self) -> Result<ReferenceType> {
        let mut out = String::new();
        self.expect_next(notation::CLASS)?;

        while self.peek_next()? != notation::END_REFERENCE {
            out.push(self.consume_next()?);
        }
        self.consume_next()?; // consume the end reference
        Ok(ReferenceType { internal_name: out })
    }

    fn parse_descriptor_type(&mut self) -> Result<DescriptorType> {
        let next = self.peek_next()?;

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
        let next = self.chars.next();

        if next.is_none() {
            return Err(anyhow!("out of chars"));
        }

        Ok(next.unwrap())
    }

    fn peek_next(&mut self) -> Result<char> {
        let next = self.chars.peek();

        if next.is_none() {
            return Err(anyhow!("out of chars"));
        }

        Ok(*next.unwrap())
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

#[derive(Clone)]
pub struct MethodDescriptor {
    pub parameters: Vec<DescriptorType>,
    pub return_type: DescriptorType,
}

impl MethodDescriptor {
    pub fn parse(descriptor: &str) -> Result<Self> {
        let mut parser = Parser::new(descriptor);

        parser.parse_method_descriptor()
    }
}

#[derive(Clone)]
pub struct FieldDescriptor {
    _type: DescriptorType,
}

impl FieldDescriptor {
    pub fn parse(descriptor: &str) -> Result<Self> {
        let mut parser = Parser::new(descriptor);

        let _type = parser.parse_descriptor_type()?;

        Ok(Self { _type })
    }
}

#[derive(PartialEq, Clone)]
pub enum DescriptorType {
    Reference(ReferenceType),
    Primitive(PrimitiveType),
    Array(ArrayType),
    Void,
}

#[derive(PartialEq, Clone)]
pub struct ArrayType {
    pub _type: Box<DescriptorType>,
    pub dimensions: u16,
}

#[derive(PartialEq, Clone)]
pub struct ReferenceType {
    pub internal_name: String,
}

#[derive(Clone)]
pub enum Descriptor {
    Field(FieldDescriptor),
    Method(MethodDescriptor),
}
pub fn test_descriptor_parsing() {
    /*
        oneArrayParam:([Ljava/lang/String;)V
        twoArrayParams:([Ljava/lang/String;[Ljava/lang/String;)V
        oneTwoDArrayParam:([[Ljava/lang/String;)V
        oneThreeDArrayParam:([[[Ljava/lang/String;)V
        oneRefParam:(Ljava/lang/String;)V
        twoRefParams:(Ljava/lang/String;Ljava/lang/String;)V
    */

    let oneArrayParam = "([Ljava/lang/String;)V";
    let twoArrayParams = "([Ljava/lang/String;[Ljava/lang/String;)V";
    let oneTwoDArrayParam = "([[Ljava/lang/String;)V";
    let oneThreeDArrayParam = "([[[Ljava/lang/String;)V";
    let oneRefParam = "(Ljava/lang/String;)V";
    let twoRefParams = "(Ljava/lang/String;Ljava/lang/String;)V";

    let _all_valid = [
        MethodDescriptor::parse(oneArrayParam).unwrap(),
        MethodDescriptor::parse(twoArrayParams).unwrap(),
        MethodDescriptor::parse(oneTwoDArrayParam).unwrap(),
        MethodDescriptor::parse(oneThreeDArrayParam).unwrap(),
        MethodDescriptor::parse(oneRefParam).unwrap(),
        MethodDescriptor::parse(twoRefParams).unwrap(),
    ];
}
