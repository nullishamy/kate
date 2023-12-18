use crate::{
    classfile::{Addressed, Resolvable},
    pool::{ConstantClass, ConstantEntry, ConstantPool, ConstantUtf8},
};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use support::bytes_ext::SafeBuf;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Addressed<ConstantUtf8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Attributes {
    pub values: Vec<Attribute>,
}

impl Attributes {
    pub fn known_attribute<T>(&self, constant_pool: &ConstantPool) -> Result<T>
    where
        T: KnownAttribute,
    {
        let mut found_attr: Option<&Attribute> = None;

        for attr in self.values.iter() {
            let name = attr.name.resolve();
            let str = name.try_string()?;

            if T::id() == str {
                found_attr = Some(attr)
            }
        }

        if found_attr.is_none() {
            return Err(anyhow!("could not locate known attribute {}", T::id()));
        }

        let bytes = found_attr.unwrap().data.clone();
        let bytes: Bytes = Bytes::copy_from_slice(&bytes);
        T::decode(bytes, constant_pool)
    }

    pub fn parse(bytes: &mut Bytes, constant_pool: &ConstantPool) -> Result<Self> {
        let length = bytes.try_get_u16()?;
        let mut attributes = Attributes {
            values: Vec::with_capacity(length.into()),
        };

        for _ in 0..length {
            let name = constant_pool.address(bytes.try_get_u16()?);
            let attr_length = bytes.try_get_u32()?;
            let mut info: Vec<u8> = Vec::new();

            for _ in 0..attr_length {
                info.push(bytes.try_get_u8()?);
            }

            attributes.values.push(Attribute { name, data: info });
        }

        Ok(attributes)
    }
}

pub trait KnownAttribute
where
    Self: Sized,
{
    fn decode(bytes: Bytes, constant_pool: &ConstantPool) -> Result<Self>;
    fn id() -> &'static str;
}

impl KnownAttribute for CodeAttribute {
    fn decode(mut bytes: Bytes, constant_pool: &ConstantPool) -> Result<Self> {
        let max_stack = bytes.try_get_u16()?;
        let max_locals = bytes.try_get_u16()?;

        let code_length = bytes.try_get_u32()?;
        let mut code: Vec<u8> = Vec::new();
        for _ in 0..code_length {
            code.push(bytes.try_get_u8()?);
        }

        let exception_length = bytes.try_get_u16()?;
        let mut exception_table: Vec<ExceptionEntry> = Vec::with_capacity(exception_length.into());
        for _ in 0..exception_length {
            exception_table.push(ExceptionEntry {
                start_pc: bytes.try_get_u16()?,
                end_pc: bytes.try_get_u16()?,
                handler_pc: bytes.try_get_u16()?,
                catch_type: constant_pool.address(bytes.try_get_u16()?),
            })
        }
        let attributes = Attributes::parse(&mut bytes, constant_pool)?;

        Ok(CodeAttribute {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        })
    }

    fn id() -> &'static str {
        "Code"
    }
}

#[derive(Debug, Clone)]
pub struct ConstantValueAttribute {
    pub value: Addressed<ConstantEntry>,
}

impl KnownAttribute for ConstantValueAttribute {
    fn decode(mut bytes: Bytes, constant_pool: &ConstantPool) -> Result<Self> {
        Ok(ConstantValueAttribute {
            value: constant_pool.address(bytes.try_get_u16()?),
        })
    }

    fn id() -> &'static str {
        "ConstantValue"
    }
}

#[derive(Debug, Clone)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionEntry>,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct ExceptionEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: Addressed<ConstantClass>,
}

pub struct StackMapTableAttribute {}
pub struct BootstrapMethodsAttribute {}
pub struct NestHostAttribute {}
pub struct NestMembersAttribute {}
pub struct PermittedSubclassesAttribute {}
