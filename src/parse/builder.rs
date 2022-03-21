use crate::parse::bytecode::ByteSize;
use crate::types::classfile::{
    AttributeInfo, AttributeInfoEntry, ConstantPoolData, ConstantPoolEntry, ConstantPoolInfo,
    ConstantPoolTag, FieldInfo, FieldInfoEntry, InterfaceInfo, MethodInfo, MethodInfoEntry,
};
use crate::types::ErrString;
use crate::ByteCode;
use bytes::Buf;

pub fn make_utf8_string(byte_code: &mut ByteCode) -> Result<ConstantPoolData, ErrString> {
    byte_code.require_size(ByteSize::U16)?;
    let length = byte_code.data().get_u16();

    let mut bytes: Vec<u8> = Vec::with_capacity(length as usize);
    let mut idx = 0;

    while idx < length {
        byte_code.require_size(ByteSize::U8)?;
        bytes.push(byte_code.data().get_u8());
        idx += 1;
    }

    let as_str = String::from_utf8(bytes.clone());

    if as_str.is_err() {
        return Err("invalid utf-8 string".to_string());
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
) -> Result<ConstantPoolData, ErrString> {
    match tag {
        ConstantPoolTag::Utf8 => make_utf8_string(byte_code),
        ConstantPoolTag::Integer => {
            byte_code.require_size(ByteSize::U32)?;
            Ok(ConstantPoolData::Integer {
                bytes: byte_code.data().get_u32(),
            })
        }
        ConstantPoolTag::Float => {
            byte_code.require_size(ByteSize::F32)?;
            Ok(ConstantPoolData::Float {
                bytes: byte_code.data().get_f32(),
            })
        }
        ConstantPoolTag::Long => {
            byte_code.require_size(ByteSize::U32)?;
            let high_bytes = byte_code.data().get_u32();

            byte_code.require_size(ByteSize::U32)?;
            let low_bytes = byte_code.data().get_u32();

            Ok(ConstantPoolData::Long {
                low_bytes,
                high_bytes,
            })
        }
        ConstantPoolTag::Double => {
            byte_code.require_size(ByteSize::F32)?;
            let high_bytes = byte_code.data().get_f32();

            byte_code.require_size(ByteSize::F32)?;
            let low_bytes = byte_code.data().get_f32();

            Ok(ConstantPoolData::Double {
                low_bytes,
                high_bytes,
            })
        }
        ConstantPoolTag::Class => {
            byte_code.require_size(ByteSize::U16)?;
            Ok(ConstantPoolData::Class {
                name_index: byte_code.data().get_u16(),
            })
        }
        ConstantPoolTag::String => {
            byte_code.require_size(ByteSize::U16)?;
            Ok(ConstantPoolData::String {
                utf8_index: byte_code.data().get_u16(),
            })
        }
        ConstantPoolTag::FieldRef => {
            byte_code.require_size(ByteSize::U16)?;
            let class_index = byte_code.data().get_u16();

            byte_code.require_size(ByteSize::U16)?;
            let name_and_type_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::FieldRef {
                class_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::MethodRef => {
            byte_code.require_size(ByteSize::U16)?;
            let class_index = byte_code.data().get_u16();

            byte_code.require_size(ByteSize::U16)?;
            let name_and_type_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::MethodRef {
                class_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::InterfaceMethodRef => {
            byte_code.require_size(ByteSize::U16)?;
            let class_index = byte_code.data().get_u16();

            byte_code.require_size(ByteSize::U16)?;
            let name_and_type_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::NameAndType => {
            byte_code.require_size(ByteSize::U16)?;
            let name_index = byte_code.data().get_u16();

            byte_code.require_size(ByteSize::U16)?;
            let descriptor_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::NameAndType {
                name_index,
                descriptor_index,
            })
        }
        ConstantPoolTag::MethodHandle => {
            byte_code.require_size(ByteSize::U8)?;
            let reference_kind = byte_code.data().get_u8();

            byte_code.require_size(ByteSize::U16)?;
            let reference_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::MethodHandle {
                reference_kind,
                reference_index,
            })
        }
        ConstantPoolTag::MethodType => {
            byte_code.require_size(ByteSize::U16)?;
            Ok(ConstantPoolData::MethodType {
                descriptor_index: byte_code.data().get_u16(),
            })
        }
        ConstantPoolTag::Dynamic => {
            byte_code.require_size(ByteSize::U16)?;
            let bootstrap_method_attr_index = byte_code.data().get_u16();

            byte_code.require_size(ByteSize::U16)?;
            let name_and_type_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::InvokeDynamic => {
            byte_code.require_size(ByteSize::U16)?;
            let bootstrap_method_attr_index = byte_code.data().get_u16();

            byte_code.require_size(ByteSize::U16)?;
            let name_and_type_index = byte_code.data().get_u16();

            Ok(ConstantPoolData::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
        ConstantPoolTag::Module => {
            byte_code.require_size(ByteSize::U16)?;
            Ok(ConstantPoolData::Module {
                name_index: byte_code.data().get_u16(),
            })
        }
        ConstantPoolTag::Package => {
            byte_code.require_size(ByteSize::U16)?;
            Ok(ConstantPoolData::Package {
                name_index: byte_code.data().get_u16(),
            })
        }
    }
}

pub fn make_const_pool(
    byte_code: &mut ByteCode,
    pool_size: u16,
) -> Result<ConstantPoolInfo, ErrString> {
    let mut const_pool = ConstantPoolInfo::new(pool_size);

    //TODO: figure out what index 0 should have in the const pool and alter this

    // -1 because the const pool is indexed from 1 -> len - 1
    while const_pool.data().len() < (pool_size - 1) as usize {
        byte_code.require_size(ByteSize::U8)?;
        let tag = ConstantPoolTag::new(byte_code.data().get_u8(), byte_code)?;
        let data = make_const_pool_data(byte_code, &tag)?;
        let entry = ConstantPoolEntry::new(tag, data);

        const_pool.data().push(entry);
    }
    Ok(const_pool)
}

pub fn make_interface_info(
    byte_code: &mut ByteCode,
    length: u16,
) -> Result<InterfaceInfo, ErrString> {
    let mut out: Vec<u16> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        byte_code.require_size(ByteSize::U16)?;
        out.push(byte_code.data().get_u16());
    }
    Ok(InterfaceInfo::new(out))
}

pub fn make_attribute_info(
    byte_code: &mut ByteCode,
    length: u16,
) -> Result<AttributeInfo, ErrString> {
    let mut out: Vec<AttributeInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        byte_code.require_size(ByteSize::U16)?;
        let attribute_name_index = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U32)?;
        let attribute_length = byte_code.data().get_u32();

        let mut attributes: Vec<u8> = Vec::with_capacity(attribute_length as usize);

        while attributes.len() < attribute_length as usize {
            byte_code.require_size(ByteSize::U8)?;
            attributes.push(byte_code.data().get_u8());
        }
        out.push(AttributeInfoEntry::new(
            attribute_name_index,
            attribute_length,
            attributes,
        ));
    }
    Ok(AttributeInfo::new(out))
}

pub fn make_field_info(byte_code: &mut ByteCode, length: u16) -> Result<FieldInfo, ErrString> {
    let mut out: Vec<FieldInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        byte_code.require_size(ByteSize::U16)?;
        let access_flags = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let name_index = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let descriptor_index = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let attributes_count = byte_code.data().get_u16();

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

pub fn make_method_info(byte_code: &mut ByteCode, length: u16) -> Result<MethodInfo, ErrString> {
    let mut out: Vec<MethodInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        byte_code.require_size(ByteSize::U16)?;
        let access_flags = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let name_index = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let descriptor_index = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
        let attributes_count = byte_code.data().get_u16();

        byte_code.require_size(ByteSize::U16)?;
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
