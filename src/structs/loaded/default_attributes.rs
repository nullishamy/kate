use crate::classfile::parse_helper::SafeBuf;
use crate::structs::loaded::constant_pool::{ConstantPool, Utf8Data};
use anyhow::Result;
use bytes::Bytes;
use enum_as_inner::EnumAsInner;
use std::rc::Rc;

#[derive(Clone)]
pub struct CodeData {
    pub name: Rc<Utf8Data>,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub attributes: Vec<CustomData>,
}

#[derive(Clone)]
pub struct ExceptionHandler {
    pub start: u16,
    pub end: u16,
    pub handler: u16,
    pub catch_type: u16,
}

impl CodeData {
    pub fn from_bytes(
        name: Rc<Utf8Data>,
        bytes: Vec<u8>,
        const_pool: &ConstantPool,
    ) -> Result<Self> {
        let mut bytes = Bytes::copy_from_slice(&bytes);

        // unsure what this is for
        let _attribute_length = bytes.try_get_u16()?;

        let max_stack = bytes.try_get_u16()?;
        let max_locals = bytes.try_get_u16()?;

        let code_length = bytes.try_get_u16()?;
        let mut code = Vec::with_capacity(code_length as usize);

        for _ in 0..code_length {
            code.push(bytes.try_get_u8()?);
        }

        let exception_length = bytes.try_get_u16()?;
        let mut exception_handlers = Vec::with_capacity(exception_length as usize);

        for _ in 0..exception_length {
            exception_handlers.push(ExceptionHandler {
                start: bytes.try_get_u16()?,
                end: bytes.try_get_u16()?,
                handler: bytes.try_get_u16()?,
                catch_type: bytes.try_get_u16()?,
            })
        }

        let attribute_count = bytes.try_get_u16()?;
        let mut attributes = Vec::with_capacity(attribute_count as usize);

        for _ in 0..attribute_count {
            let name = Rc::clone(&const_pool.utf8(bytes.try_get_u16()? as usize)?);
            let data_length = bytes.try_get_u16()?;
            let mut data = Vec::with_capacity(data_length as usize);

            for _ in 0..data_length {
                data.push(bytes.try_get_u8()?);
            }

            attributes.push(CustomData { name, data })
        }

        Ok(Self {
            name,
            max_stack,
            max_locals,
            code,
            exception_handlers,
            attributes,
        })
    }
}

#[derive(Clone)]
pub struct CustomData {
    pub name: Rc<Utf8Data>,
    pub data: Vec<u8>,
}

impl CustomData {
    pub fn from_bytes(bytes: Vec<u8>, const_pool: &ConstantPool) -> Result<Self> {
        let mut bytes = Bytes::copy_from_slice(&bytes);
        let name = Rc::clone(&const_pool.utf8(bytes.try_get_u16()? as usize)?);
        let data_length = bytes.try_get_u16()?;
        let mut data = Vec::with_capacity(data_length as usize);

        for _ in 0..data_length {
            data.push(bytes.try_get_u8()?);
        }

        Ok(CustomData { name, data })
    }
}

#[derive(Clone, EnumAsInner)]
pub enum AttributeEntry {
    Code(CodeData),
    Custom(CustomData),
}

impl AttributeEntry {
    pub fn name(&self) -> Rc<Utf8Data> {
        match self {
            AttributeEntry::Code(data) => Rc::clone(&data.name),
            AttributeEntry::Custom(data) => Rc::clone(&data.name),
        }
    }
}
