use std::fs::File;
use std::io::{ErrorKind, Read};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use tracing::{debug, info, warn};

use crate::classfile::parse_helper::{
    parse_attribute_info, parse_const_pool, parse_field_info, parse_interface_info,
    parse_method_info, SafeBuf,
};
use crate::structs::raw::classfile::{
    RawClassFile, MAGIC, MAX_SUPPORTED_MAJOR, MAX_SUPPORTED_MINOR,
};

pub struct ClassFileParser {
    pub name: String,
    pub bytes: Bytes,
}

impl ClassFileParser {
    pub fn from_path(path: String) -> Result<Self> {
        info!("opening classfile '{}' for parsing from path", path);

        let buffer = ClassFileParser::bytes(path.clone())?;
        Ok(ClassFileParser::from_bytes(path, buffer))
    }

    pub fn from_bytes(name: String, bytes: Vec<u8>) -> Self {
        debug!("creating parser from bytes for class '{}'", name);

        Self {
            name,
            bytes: Bytes::copy_from_slice(&bytes),
        }
    }

    pub fn bytes(path: String) -> Result<Vec<u8>> {
        info!("opening classfile '{}' to read bytes", path);

        let path = if path.starts_with("java/") {
            debug!("java stdlib detected, altering path");
            format!("src/stdlib/{}", path)
        } else {
            path
        };
        let path_with_ext = format!("{}.class", path);

        debug!("loading bytes from '{}'", path_with_ext);

        let mut file_handle = match File::open(&path_with_ext) {
            Ok(handle) => handle,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    let handle = File::open(&path);

                    if let Err(e) = handle {
                        return Err(anyhow!(
                            "failed to open file '{}' because of error {}",
                            path,
                            e
                        ));
                    }

                    handle.unwrap()
                }
                _ => {
                    return Err(anyhow!(
                        "failed to open file '{}' because of error {}",
                        path,
                        err
                    ))
                }
            },
        };

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

    pub fn parse(&mut self) -> Result<RawClassFile> {
        debug!("parsing bytes for class '{}'", self.name);

        let magic = self.bytes.try_get_u32()?;

        debug!("got {magic} as the magic value");

        if magic != MAGIC {
            return Err(anyhow!("magic value not present or not matching"));
        }

        let minor = self.bytes.try_get_u16()?;

        debug!("got {minor} as the minor version");

        if minor > MAX_SUPPORTED_MINOR {
            return Err(anyhow!("minor version not supported"));
        }

        let major = self.bytes.try_get_u16()?;

        debug!("got {major} as the major version");

        if major > MAX_SUPPORTED_MAJOR {
            return Err(anyhow!("major version not supported"));
        }

        let const_pool_count = self.bytes.try_get_u16()?;

        debug!("const pool has {const_pool_count} entries listed");
        let const_pool_info = parse_const_pool(self, const_pool_count)?;
        debug!(
            "successfully parsed constant pool, resulting vec has {} items",
            const_pool_info.len()
        );

        let access_flags = self.bytes.try_get_u16()?;
        debug!("got access flags {:b}", access_flags);

        let this_class = self.bytes.try_get_u16()?;
        debug!("got this_class index {this_class}");

        let super_class = self.bytes.try_get_u16()?;
        debug!("got super_class index {super_class}");

        let interface_count = self.bytes.try_get_u16()?;
        debug!("class has {interface_count} interfaces");
        let interface_info = parse_interface_info(self, interface_count)?;
        debug!(
            "successfully parsed interfaces, resulting vec has {} items",
            interface_info.len()
        );

        let field_count = self.bytes.try_get_u16()?;
        debug!("class has {field_count} fields");
        let field_info = parse_field_info(self, field_count)?;
        debug!(
            "successfully parsed fields, resulting vec has {} items",
            field_info.len()
        );

        let method_count = self.bytes.try_get_u16()?;
        debug!("class has {method_count} methods");
        let method_info = parse_method_info(self, method_count)?;
        debug!(
            "successfully parsed methods, resulting vec has {} items",
            method_info.len()
        );

        let attribute_count = self.bytes.try_get_u16()?;
        debug!("class has {attribute_count} attributes");
        let attribute_info = parse_attribute_info(self, attribute_count)?;
        debug!(
            "successfully parsed attributes, resulting vec has {} items",
            attribute_info.len()
        );

        if !self.bytes.is_empty() {
            return Err(anyhow!(
                "invalid classfile detected, trailing bytes were present"
            ));
        }

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
