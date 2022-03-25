use crate::classfile::parse_helper::{
    parse_attribute_info, parse_const_pool, parse_field_info, parse_interface_info,
    parse_method_info, SafeBuf,
};
use crate::structs::raw::classfile::{
    RawClassFile, MAGIC, MAX_SUPPORTED_MAJOR, MAX_SUPPORTED_MINOR,
};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::fs::File;
use std::io::Read;

pub struct ClassFileParser {
    pub name: String,
    pub bytes: Bytes,
}

impl ClassFileParser {
    pub fn from_path(path: String) -> Result<Self> {
        let buffer = ClassFileParser::bytes(path.to_owned())?;
        let bytes = Bytes::copy_from_slice(&buffer);

        Ok(Self { name: path, bytes })
    }

    pub fn bytes(path: String) -> Result<Vec<u8>> {
        let file_handle = File::open(&path);

        if let Err(e) = file_handle {
            return Err(anyhow!(
                "failed to open file {} because of error {}",
                &path,
                e
            ));
        }

        let mut file_handle = file_handle.unwrap();
        let mut buffer = Vec::new();

        if let Err(e) = file_handle.read_to_end(&mut buffer) {
            return Err(anyhow!(
                "failed to open file '{}' because of error {}",
                &path,
                e
            ));
        }

        Ok(buffer)
    }

    pub fn from_bytes(name: String, bytes: Vec<u8>) -> Self {
        Self {
            name,
            bytes: Bytes::copy_from_slice(&bytes),
        }
    }

    pub fn parse(&mut self) -> Result<RawClassFile> {
        let magic = self.bytes.try_get_u32()?;

        if magic != MAGIC {
            return Err(anyhow!("magic value not present or not matching"));
        }

        let minor = self.bytes.try_get_u16()?;

        if minor > MAX_SUPPORTED_MINOR {
            return Err(anyhow!("minor version not supported"));
        }

        let major = self.bytes.try_get_u16()?;

        if major > MAX_SUPPORTED_MAJOR {
            return Err(anyhow!("major version not supported"));
        }

        let const_pool_count = self.bytes.try_get_u16()?;
        let const_pool_info = parse_const_pool(self, const_pool_count)?;

        let access_flags = self.bytes.try_get_u16()?;

        let this_class = self.bytes.try_get_u16()?;

        let super_class = self.bytes.try_get_u16()?;

        let interface_count = self.bytes.try_get_u16()?;
        let interface_info = parse_interface_info(self, interface_count)?;

        let field_count = self.bytes.try_get_u16()?;
        let field_info = parse_field_info(self, field_count)?;

        let method_count = self.bytes.try_get_u16()?;
        let method_info = parse_method_info(self, method_count)?;

        let attribute_count = self.bytes.try_get_u16()?;
        let attribute_info = parse_attribute_info(self, attribute_count)?;

        Ok(RawClassFile {
            magic,
            minor_version: minor,
            major_version: major,
            const_pool_count,
            const_pool_info,
            access_flags,
            this_class,
            super_class,
            interface_count,
            interface_info,
            field_count,
            field_info,
            method_count,
            method_info,
            attribute_count,
            attribute_info,
        })
    }
}
