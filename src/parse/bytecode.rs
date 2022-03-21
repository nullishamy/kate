use crate::ErrString;
use bytes::Bytes;
use std::borrow::BorrowMut;

pub struct ByteCode<'a> {
    data: Bytes,
    file_name: &'a str,
}

pub enum ByteSize {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F8,
    F16,
    F32,
    F64,
    F128,
}

impl ByteSize {
    pub fn raw(&self) -> usize {
        match self {
            ByteSize::U8 => 1,
            ByteSize::U16 => 2,
            ByteSize::U32 => 4,
            ByteSize::U64 => 8,
            ByteSize::U128 => 16,
            ByteSize::I8 => 1,
            ByteSize::I16 => 2,
            ByteSize::I32 => 4,
            ByteSize::I64 => 8,
            ByteSize::I128 => 16,
            ByteSize::F8 => 1,
            ByteSize::F16 => 2,
            ByteSize::F32 => 4,
            ByteSize::F64 => 8,
            ByteSize::F128 => 16,
        }
    }
}

impl<'a> ByteCode<'a> {
    pub fn new(data: Vec<u8>, file_name: &'a str) -> ByteCode<'a> {
        Self {
            data: Bytes::copy_from_slice(&data),
            file_name,
        }
    }

    pub fn data(&mut self) -> &mut Bytes {
        self.data.borrow_mut()
    }

    pub fn file_name(&self) -> &'a str {
        self.file_name
    }

    pub fn require_size(&mut self, size: ByteSize) -> Result<(), ErrString> {
        // length decreases as bytes are consumed, so we can 0 check for safety
        if (self.data.len() as isize) - (size.raw() as isize) < 0 {
            return Err("no more bytes".to_string());
        }

        Ok(())
    }
}
