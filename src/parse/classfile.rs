use crate::parse::builder::{
    make_attribute_info, make_const_pool, make_field_info, make_interface_info, make_method_info,
};
use crate::parse::bytecode::ByteSize;
use crate::types::classfile::{
    AttributeInfo, ConstantPoolEntry, ConstantPoolInfo, FieldInfo, InterfaceInfo, MethodInfo,
    MethodInfoEntry,
};
use crate::types::flag::ClassFileAccessFlags;
use crate::types::ErrString;
use crate::ByteCode;
use bytes::Buf;
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
    pub fn new(byte_code: &mut ByteCode, class_file: &str) -> Result<Self, ErrString> {
        byte_code.require_size(ByteSize::U32)?;

        let magic = byte_code.data().get_u32();

        if magic != MAGIC {
            return Err("magic value not present or not matching".to_string());
        }

        byte_code.require_size(ByteSize::U16)?;
        let minor = byte_code.data().get_u16();

        if minor > MAX_SUPPORTED_MINOR {
            return Err("minor version not supported".to_string());
        }

        byte_code.require_size(ByteSize::U16)?;
        let major = byte_code.data().get_u16();

        if major > MAX_SUPPORTED_MAJOR {
            return Err("major version not supported".to_string());
        }

        byte_code.require_size(ByteSize::U16)?;
        let const_pool_count = byte_code.data().get_u16();
        let const_pool_info = make_const_pool(byte_code, const_pool_count)?;

        byte_code.require_size(ByteSize::U16)?;
        let access_flags = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let this_class = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let super_class = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let interface_count = byte_code.data().get_u16();
        let interface_info = make_interface_info(byte_code, interface_count)?;

        byte_code.require_size(ByteSize::U16)?;
        let field_count = byte_code.data().get_u16();
        let field_info = make_field_info(byte_code, field_count)?;

        byte_code.require_size(ByteSize::U16)?;
        let method_count = byte_code.data().get_u16();
        let method_info = make_method_info(byte_code, method_count)?;

        byte_code.require_size(ByteSize::U16)?;
        let attribute_count = byte_code.data().get_u16();
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

    pub fn prepare(&mut self) -> Result<ClassFile, ErrString> {
        //TODO: prepare the classfile for validation
        /*
           1) convert const pool into a hashmap
        */
        let mut const_pool: HashMap<usize, &ConstantPoolEntry> =
            HashMap::with_capacity(self.const_pool_count as usize);

        for (idx, entry) in self.const_pool_info.data().iter().enumerate() {
            if const_pool.contains_key(&idx) {
                return Err(format!("duplicate const pool entry {}", idx));
            }

            const_pool.insert(idx, entry);
        }

        let meta = ClassFileMetaData {
            minor_version: self.minor_version,
            major_version: self.major_version,
        };

        let access_flags = ClassFileAccessFlags::from_bits(self.access_flags)?;

        Ok(ClassFile {
            const_pool,
            meta,
            access_flags,
        })
    }
}

pub struct ClassFileMetaData {
    minor_version: u16,
    major_version: u16,
}

pub struct ClassFile<'a> {
    const_pool: HashMap<usize, &'a ConstantPoolEntry>,
    meta: ClassFileMetaData,
    access_flags: ClassFileAccessFlags,
}

impl<'a> ClassFile<'a> {
    pub fn validate(self) -> Result<ClassFile<'a>, ErrString> {
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
}
