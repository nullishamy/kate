use anyhow::{anyhow, Result};
use tracing::trace;

use crate::structs::raw::attribute::AttributeEntry;
use crate::structs::raw::constant_pool::InterfaceMethodRefData;
use crate::structs::raw::constant_pool::{
    ClassData, Data, DoubleData, FieldRefData, FloatData, IntegerData, LongData, MethodRefData,
    NameAndTypeData, PoolEntry, StringData, Tag, Utf8Data,
};
use crate::structs::raw::field::FieldEntry;
use crate::structs::raw::method::MethodEntry;
use crate::ClassFileParser;

pub fn parse_utf8_string(parser: &mut ClassFileParser) -> Result<Utf8Data> {
    let length = parser.bytes.try_get_u16()?;

    let mut bytes = Vec::with_capacity(length as usize);

    for _ in 0..length {
        bytes.push(parser.bytes.try_get_u8()?);
    }

    trace!(
        "parsed utf has string {:?}",
        String::from_utf8(bytes.clone())?
    );

    Ok(Utf8Data { length, bytes })
}

pub fn parse_integer_data(parser: &mut ClassFileParser) -> Result<IntegerData> {
    Ok(IntegerData {
        bytes: parser.bytes.try_get_u32()?,
    })
}

pub fn parse_float_data(parser: &mut ClassFileParser) -> Result<FloatData> {
    Ok(FloatData {
        bytes: parser.bytes.try_get_f32()?,
    })
}

pub fn parse_long_data(parser: &mut ClassFileParser) -> Result<LongData> {
    Ok(LongData {
        low_bytes: parser.bytes.try_get_u32()?,
        high_bytes: parser.bytes.try_get_u32()?,
    })
}

pub fn parse_double_data(parser: &mut ClassFileParser) -> Result<DoubleData> {
    Ok(DoubleData {
        low_bytes: parser.bytes.try_get_f32()?,
        high_bytes: parser.bytes.try_get_f32()?,
    })
}

pub fn parse_class_data(parser: &mut ClassFileParser) -> Result<ClassData> {
    Ok(ClassData {
        name_index: parser.bytes.try_get_u16()?,
    })
}

pub fn parse_string_data(parser: &mut ClassFileParser) -> Result<StringData> {
    Ok(StringData {
        utf8_index: parser.bytes.try_get_u16()?,
    })
}

pub fn parse_method_ref_data(parser: &mut ClassFileParser) -> Result<MethodRefData> {
    Ok(MethodRefData {
        class_index: parser.bytes.try_get_u16()?,
        name_and_type_index: parser.bytes.try_get_u16()?,
    })
}

pub fn parse_field_ref_data(parser: &mut ClassFileParser) -> Result<FieldRefData> {
    Ok(FieldRefData {
        class_index: parser.bytes.try_get_u16()?,
        name_and_type_index: parser.bytes.try_get_u16()?,
    })
}

pub fn parse_name_and_type_data(parser: &mut ClassFileParser) -> Result<NameAndTypeData> {
    Ok(NameAndTypeData {
        name_index: parser.bytes.try_get_u16()?,
        descriptor_index: parser.bytes.try_get_u16()?,
    })
}

pub fn parse_interface_method_ref_data(
    parser: &mut ClassFileParser,
) -> Result<InterfaceMethodRefData> {
    Ok(InterfaceMethodRefData {
        class_index: parser.bytes.try_get_u16()?,
        name_and_type_index: parser.bytes.try_get_u16()?,
    })
}

pub fn parse_pool_data(parser: &mut ClassFileParser, tag: &Tag) -> Result<Data> {
    Ok(match tag {
        Tag::Utf8 => Data::Utf8(parse_utf8_string(parser)?),
        Tag::Integer => Data::Integer(parse_integer_data(parser)?),
        Tag::Float => Data::Float(parse_float_data(parser)?),
        Tag::Long => Data::Long(parse_long_data(parser)?),
        Tag::Double => Data::Double(parse_double_data(parser)?),
        Tag::Class => Data::Class(parse_class_data(parser)?),
        Tag::String => Data::String(parse_string_data(parser)?),
        Tag::FieldRef => Data::FieldRef(parse_field_ref_data(parser)?),
        Tag::MethodRef => Data::MethodRef(parse_method_ref_data(parser)?),
        Tag::InterfaceMethodRef => {
            Data::InterfaceMethodRef(parse_interface_method_ref_data(parser)?)
        }
        Tag::NameAndType => Data::NameAndType(parse_name_and_type_data(parser)?),
        Tag::MethodHandle => todo!(),
        Tag::MethodType => todo!(),
        Tag::Dynamic => todo!(),
        Tag::InvokeDynamic => todo!(),
        Tag::Module => todo!(),
        Tag::Package => todo!(),
    })
}

pub fn parse_const_pool(
    parser: &mut ClassFileParser,
    pool_size: u16,
) -> Result<Vec<Option<PoolEntry>>> {
    let mut pool_data: Vec<Option<PoolEntry>> = Vec::with_capacity(pool_size as usize);

    pool_data.push(None);

    // pool is indexed from 1 -> size -1
    // we add "None" at the start to account for this
    // while loop because some entries take up 2 slots
    // thus, a ranged loop would not work

    let mut i = 1;
    while pool_data.len() < pool_size as usize {
        trace!("parsing const pool entry {i}");
        let tag_byte = parser.bytes.try_get_u8()?;

        trace!("const pool entry has tag byte {:?}", tag_byte);
        let tag = Tag::from_tag_byte(tag_byte)?;

        trace!("const pool entry has tag {:?}", tag);
        let data = parse_pool_data(parser, &tag)?;
        let entry = PoolEntry { tag, data };

        trace!("pushing entry {:?}", entry);

        match tag {
            Tag::Long => {
                trace!("special casing Tag::Long by pushing 2 entries");
                pool_data.push(Some(entry));
                pool_data.push(None);
                i += 2;
            }
            Tag::Double => {
                trace!("special casing Tag::Double by pushing 2 entries");
                pool_data.push(Some(entry));
                pool_data.push(None);
                i += 2;
            }
            _ => {
                pool_data.push(Some(entry));
                i += 1;
            }
        }
    }

    Ok(pool_data)
}

pub fn parse_interface_info(parser: &mut ClassFileParser, length: u16) -> Result<Vec<u16>> {
    let mut out = Vec::with_capacity(length as usize);
    for _ in 0..length {
        out.push(parser.bytes.try_get_u16()?);
    }
    Ok(out)
}

pub fn parse_attribute_info(
    parser: &mut ClassFileParser,
    length: u16,
) -> Result<Vec<AttributeEntry>> {
    let mut out: Vec<AttributeEntry> = Vec::with_capacity(length as usize);

    for _ in 0..length {
        let attribute_name_index = parser.bytes.try_get_u16()?;
        let attribute_length = parser.bytes.try_get_u32()?;

        let mut attribute_data: Vec<u8> = Vec::with_capacity(attribute_length as usize);
        while attribute_data.len() < attribute_length as usize {
            attribute_data.push(parser.bytes.try_get_u8()?);
        }

        out.push(AttributeEntry {
            attribute_name_index,
            attribute_length,
            attribute_data,
        });
    }

    Ok(out)
}

pub fn parse_field_info(parser: &mut ClassFileParser, length: u16) -> Result<Vec<FieldEntry>> {
    let mut out = Vec::with_capacity(length as usize);

    for _ in 0..length {
        let access_flags = parser.bytes.try_get_u16()?;
        let name_index = parser.bytes.try_get_u16()?;
        let descriptor_index = parser.bytes.try_get_u16()?;
        let attributes_count = parser.bytes.try_get_u16()?;
        let attribute_info = parse_attribute_info(parser, attributes_count)?;

        out.push(FieldEntry {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        });
    }

    Ok(out)
}

pub fn parse_method_info(parser: &mut ClassFileParser, length: u16) -> Result<Vec<MethodEntry>> {
    let mut out = Vec::with_capacity(length as usize);
    for _ in 0..length {
        let access_flags = parser.bytes.try_get_u16()?;
        let name_index = parser.bytes.try_get_u16()?;
        let descriptor_index = parser.bytes.try_get_u16()?;
        let attributes_count = parser.bytes.try_get_u16()?;
        let attribute_info = parse_attribute_info(parser, attributes_count)?;

        out.push(MethodEntry {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        });
    }

    Ok(out)
}

/**
This macro builds a set of `try_get_{number_type}` functions for safe reading of
bytes from a Bytes object. They return Result<T> instead of panicking
 */
macro_rules! impl_safebuf {
    ( $($type:ty),* ) => {
        pub trait SafeBuf: bytes::Buf {
            paste::paste! {
                $(
                fn [<try_get_ $type>](&mut self) -> anyhow::Result<$type>{
                    if self.remaining() >= std::mem::size_of::<$type>() {
                        Ok(self.[<get_ $type>]())
                    } else {
                        Err(anyhow!("out of bytes"))
                    }
                }
                )*
            }
        }

        impl<T: bytes::Buf> SafeBuf for T { }
    }
}

impl_safebuf!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
