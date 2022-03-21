use crate::parse::builder::{
    make_attribute_info, make_const_pool, make_field_info, make_interface_info, make_method_info,
};
use crate::parse::bytecode::SafeBuf;
use crate::types::classfile::{
    AttributeInfo, ConstantPoolData, ConstantPoolEntry, ConstantPoolInfo, FieldInfo, InterfaceInfo,
    MethodInfo,
};
use crate::types::flag::ClassFileAccessFlags;
use crate::ByteCode;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub const MAGIC: u32 = 0xCAFEBABE;
pub const MAX_SUPPORTED_MAJOR: u16 = 61;
pub const MAX_SUPPORTED_MINOR: u16 = 0;

pub struct RawClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    const_pool_count: u16,
    const_pool_info: ConstantPoolInfo,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interface_count: u16,
    interface_info: InterfaceInfo,
    field_count: u16,
    field_info: FieldInfo,
    method_count: u16,
    method_info: MethodInfo,
    attribute_count: u16,
    attribute_info: AttributeInfo,
}

impl RawClassFile {
    pub fn new(byte_code: &mut ByteCode, class_file: &str) -> Result<Self> {
        let magic = byte_code.data().try_get_u32()?;

        if magic != MAGIC {
            return Err(anyhow!("magic value not present or not matching"));
        }

        let minor = byte_code.data().try_get_u16()?;

        if minor > MAX_SUPPORTED_MINOR {
            return Err(anyhow!("minor version not supported"));
        }

        let major = byte_code.data().try_get_u16()?;

        if major > MAX_SUPPORTED_MAJOR {
            return Err(anyhow!("major version not supported"));
        }

        let const_pool_count = byte_code.data().try_get_u16()?;
        let const_pool_info = make_const_pool(byte_code, const_pool_count)?;

        let access_flags = byte_code.data().try_get_u16()?;

        let this_class = byte_code.data().try_get_u16()?;

        let super_class = byte_code.data().try_get_u16()?;

        let interface_count = byte_code.data().try_get_u16()?;
        let interface_info = make_interface_info(byte_code, interface_count)?;

        let field_count = byte_code.data().try_get_u16()?;
        let field_info = make_field_info(byte_code, field_count)?;

        let method_count = byte_code.data().try_get_u16()?;
        let method_info = make_method_info(byte_code, method_count)?;

        let attribute_count = byte_code.data().try_get_u16()?;
        let attribute_info = make_attribute_info(byte_code, attribute_count)?;

        Ok(Self {
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

    pub fn prepare(&mut self) -> Result<ClassFile> {
        //TODO: prepare the classfile for validation

        let meta = ClassFileMetaData {
            minor_version: self.minor_version,
            major_version: self.major_version,
        };

        let access_flags = ClassFileAccessFlags::from_bits(self.access_flags)?;

        Ok(ClassFile {
            meta,
            access_flags,
            const_pool: &self.const_pool_info,
            interfaces: &self.interface_info,
            fields: &self.field_info,
            methods: &self.method_info,
            attributes: &self.attribute_info,
        })
    }
}

pub struct ClassFileMetaData {
    minor_version: u16,
    major_version: u16,
}

pub struct ClassFile<'a> {
    meta: ClassFileMetaData,
    access_flags: ClassFileAccessFlags,

    // take references because this could be expensive to copy
    // and we already have a lifetime setup here
    const_pool: &'a ConstantPoolInfo,
    interfaces: &'a InterfaceInfo,
    fields: &'a FieldInfo,
    methods: &'a MethodInfo,
    attributes: &'a AttributeInfo,
}

impl<'a> ClassFile<'a> {
    fn validate_const_pool_indexes(&self) -> Result<&ClassFile<'a>> {
        for entry in self.const_pool.data().values() {
            let is_valid = match (*entry).data() {
                ConstantPoolData::Class { name_index } => self.const_pool.has(*name_index),
                ConstantPoolData::String { utf8_index } => self.const_pool.has(*utf8_index),
                ConstantPoolData::FieldRef {
                    class_index,
                    name_and_type_index,
                } => {
                    self.const_pool.has(*class_index) && self.const_pool().has(*name_and_type_index)
                }
                ConstantPoolData::MethodRef {
                    class_index,
                    name_and_type_index,
                } => {
                    self.const_pool.has(*class_index) && self.const_pool().has(*name_and_type_index)
                }
                ConstantPoolData::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                } => {
                    self.const_pool.has(*class_index) && self.const_pool().has(*name_and_type_index)
                }
                ConstantPoolData::NameAndType {
                    name_index,
                    descriptor_index,
                } => self.const_pool.has(*name_index) && self.const_pool().has(*descriptor_index),
                ConstantPoolData::MethodHandle {
                    reference_index, ..
                } => self.const_pool.has(*reference_index),
                ConstantPoolData::MethodType {
                    descriptor_index, ..
                } => self.const_pool().has(*descriptor_index),
                ConstantPoolData::Dynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                } => {
                    self.const_pool.has(*bootstrap_method_attr_index)
                        && self.const_pool().has(*name_and_type_index)
                }
                ConstantPoolData::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                } => {
                    self.const_pool.has(*bootstrap_method_attr_index)
                        && self.const_pool().has(*name_and_type_index)
                }
                ConstantPoolData::Module { name_index } => self.const_pool.has(*name_index),
                ConstantPoolData::Package { name_index } => self.const_pool.has(*name_index),
                _ => true, // for entries that have no indexes in them are implicitly valid
            };

            if !is_valid {
                return Err(anyhow!("constant pool indexes are incomplete"));
            }
        }
        Ok(self)
    }

    pub fn validate(&self) -> Result<&ClassFile<'a>> {
        self.validate_const_pool_indexes()?;
        //TODO: validate the classfile
        /*
            1) type check
            2) ensure final classes are not subclassed
            3) ensure final methods are not overwritten
            4) ensure all classes have a superclass
            5) ensure there are no remaining bytes after parsing
            6) ensure all const pool references are valid
        */

        Ok(self)
    }

    pub fn access_flags(&self) -> &ClassFileAccessFlags {
        &self.access_flags
    }

    pub fn const_pool(&self) -> &ConstantPoolInfo {
        self.const_pool
    }

    pub fn attributes(&self) -> &AttributeInfo {
        self.attributes
    }
    
    pub fn methods(&self) -> &MethodInfo {
        self.methods
    }
}
