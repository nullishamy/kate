use crate::parse::bytecode::SafeBuf;
use crate::types::classfile::{
    AttributeInfo, AttributeInfoEntry, ConstantPoolData, ConstantPoolEntry, ConstantPoolInfo,
    ConstantPoolTag, FieldInfo, FieldInfoEntry, InterfaceInfo, MethodInfo, MethodInfoEntry,
};
use crate::types::ErrString;
use crate::ByteCode;
use anyhow::{anyhow, Result};

pub fn make_utf8_string(byte_code: &mut ByteCode) -> Result<ConstantPoolData> {
    let length = byte_code.data().try_get_u16()?;

    let mut bytes: Vec<u8> = Vec::with_capacity(length as usize);
    let mut idx = 0;

    while idx < length {
        bytes.push(byte_code.data().try_get_u8()?);
        idx += 1;
    }

    let as_str = String::from_utf8(bytes.clone());

    if as_str.is_err() {
        return Err(anyhow!("invalid utf-8 string"));
    }

    Ok(ConstantPoolData::Utf8 {
        bytes,
        length,
        as_str: as_str.unwrap(),
    })
}

pub fn make_const_pool_data(
    byte_code: &mut ByteCode,
    tag: &ConstantPoolTag,
) -> Result<ConstantPoolData> {
    match tag {
        ConstantPoolTag::Utf8 => make_utf8_string(byte_code),
        ConstantPoolTag::Integer => Ok(ConstantPoolData::Integer {
            bytes: byte_code.data().try_get_u32()?,
        }),
        ConstantPoolTag::Float => Ok(ConstantPoolData::Float {
            bytes: byte_code.data().try_get_f32()?,
        }),
        ConstantPoolTag::Long => {
            let high_bytes = byte_code.data().try_get_u32()?;
            let low_bytes = byte_code.data().try_get_u32()?;

            Ok(ConstantPoolData::Long {
                low_bytes,
                high_bytes,
            })
        }
        ConstantPoolTag::Double => {
            let high_bytes = byte_code.data().try_get_f32()?;
            let low_bytes = byte_code.data().try_get_f32()?;

            Ok(ConstantPoolData::Double {
                low_bytes,
                high_bytes,
            })
        }
        ConstantPoolTag::Class => Ok(ConstantPoolData::Class {
            name_index: byte_code.data().try_get_u16()?,
        }),
        ConstantPoolTag::String => Ok(ConstantPoolData::String {
            utf8_index: byte_code.data().try_get_u16()?,
        }),
        ConstantPoolTag::FieldRef => {
            let class_index = byte_code.data().try_get_u16()?;

            let name_and_type_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::FieldRef {
                class_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::MethodRef => {
            let class_index = byte_code.data().try_get_u16()?;

            let name_and_type_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::MethodRef {
                class_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::InterfaceMethodRef => {
            let class_index = byte_code.data().try_get_u16()?;

            let name_and_type_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::NameAndType => {
            let name_index = byte_code.data().try_get_u16()?;

            let descriptor_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::NameAndType {
                name_index,
                descriptor_index,
            })
        }
        ConstantPoolTag::MethodHandle => {
            let reference_kind = byte_code.data().try_get_u8()?;

            let reference_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::MethodHandle {
                reference_kind,
                reference_index,
            })
        }
        ConstantPoolTag::MethodType => Ok(ConstantPoolData::MethodType {
            descriptor_index: byte_code.data().try_get_u16()?,
        }),
        ConstantPoolTag::Dynamic => {
            let bootstrap_method_attr_index = byte_code.data().try_get_u16()?;

            let name_and_type_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::InvokeDynamic => {
            let bootstrap_method_attr_index = byte_code.data().try_get_u16()?;

            let name_and_type_index = byte_code.data().try_get_u16()?;

            Ok(ConstantPoolData::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::Module => Ok(ConstantPoolData::Module {
            name_index: byte_code.data().try_get_u16()?,
        }),
        ConstantPoolTag::Package => Ok(ConstantPoolData::Package {
            name_index: byte_code.data().try_get_u16()?,
        }),
    }
}

pub fn make_const_pool(byte_code: &mut ByteCode, pool_size: u16) -> Result<ConstantPoolInfo> {
    let mut const_pool = ConstantPoolInfo::new(pool_size);

    //TODO: figure out what index 0 should have in the const pool and alter this

    // -1 because the const pool is indexed from 1 -> len - 1
    while const_pool.data().len() < (pool_size - 1) as usize {
        let tag = ConstantPoolTag::new(byte_code.data().try_get_u8()?, byte_code)?;
        let data = make_const_pool_data(byte_code, &tag)?;
        let entry = ConstantPoolEntry::new(tag, data);

        const_pool.data().push(entry);
    }
    Ok(const_pool)
}

pub fn make_interface_info(byte_code: &mut ByteCode, length: u16) -> Result<InterfaceInfo> {
    let mut out: Vec<u16> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        out.push(byte_code.data().try_get_u16()?);
    }
    Ok(InterfaceInfo::new(out))
}

pub fn make_attribute_info(byte_code: &mut ByteCode, length: u16) -> Result<AttributeInfo> {
    let mut out: Vec<AttributeInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        let attribute_name_index = byte_code.data().try_get_u16()?;

        let attribute_length = byte_code.data().try_get_u32()?;

        let mut attributes: Vec<u8> = Vec::with_capacity(attribute_length as usize);

        while attributes.len() < attribute_length as usize {
            attributes.push(byte_code.data().try_get_u8()?);
        }
        out.push(AttributeInfoEntry::new(
            attribute_name_index,
            attribute_length,
            attributes,
        ));
    }
    Ok(AttributeInfo::new(out))
}

pub fn make_field_info(byte_code: &mut ByteCode, length: u16) -> Result<FieldInfo> {
    let mut out: Vec<FieldInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        let access_flags = byte_code.data().try_get_u16()?;

        let name_index = byte_code.data().try_get_u16()?;

        let descriptor_index = byte_code.data().try_get_u16()?;

        let attributes_count = byte_code.data().try_get_u16()?;

        let attribute_info = make_attribute_info(byte_code, attributes_count)?;

        out.push(FieldInfoEntry::new(
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        ))
    }

    Ok(FieldInfo::new(out))
}

pub fn make_method_info(byte_code: &mut ByteCode, length: u16) -> Result<MethodInfo> {
    let mut out: Vec<MethodInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        let access_flags = byte_code.data().try_get_u16()?;

        let name_index = byte_code.data().try_get_u16()?;

        let descriptor_index = byte_code.data().try_get_u16()?;

        let attributes_count = byte_code.data().try_get_u16()?;

        let attribute_info = make_attribute_info(byte_code, attributes_count)?;

        out.push(MethodInfoEntry::new(
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        ))
    }

    Ok(MethodInfo::new(out))
}
