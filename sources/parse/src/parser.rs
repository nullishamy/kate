extern crate anyhow;
extern crate bytes;

use self::anyhow::{anyhow, Result};
use self::bytes::Bytes;

use crate::attributes::Attributes;
use crate::classfile::{
    Addressed, ClassFile, Field, Fields, Interfaces, MetaData, Method, Methods,
};
use crate::constants::MAGIC;
use crate::flags::{ClassFileAccessFlags, FieldAccessFlags, MethodAccessFlags};
use crate::pool::{
    ConstantClass, ConstantDouble, ConstantDynamic, ConstantEntry, ConstantField, ConstantFloat,
    ConstantInteger, ConstantInterfaceMethod, ConstantInvokeDynamic, ConstantLong, ConstantMethod,
    ConstantMethodHandle, ConstantMethodType, ConstantNameAndType, ConstantPool, ConstantString,
    ConstantTag, ConstantUtf8,
};
use crate::result::ParseResult;
use support::bytes_ext::SafeBuf;

pub struct Parser {
    bytes: Bytes,
}

impl Parser {
    pub fn new(data: &[u8]) -> Self {
        Self {
            bytes: Bytes::copy_from_slice(data),
        }
    }

    fn parse_constant_pool(&mut self) -> Result<ConstantPool> {
        let length = self.bytes.try_get_u16()?;
        let mut pool = ConstantPool::new();

        let mut i = 0;
        while i < (length - 1) {
            let tag = ConstantTag::from_tag(self.bytes.try_get_u8()?);
            let entry = match tag {
                ConstantTag::Class => ConstantEntry::Class(ConstantClass {
                    tag: ConstantTag::Class,
                    name: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::Field => ConstantEntry::Field(ConstantField {
                    tag: ConstantTag::Field,
                    class: pool.address(self.bytes.try_get_u16()?),
                    name_and_type: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::Method => ConstantEntry::Method(ConstantMethod {
                    tag: ConstantTag::Method,
                    class: pool.address(self.bytes.try_get_u16()?),
                    name_and_type: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::InterfaceMethod => {
                    ConstantEntry::InterfaceMethod(ConstantInterfaceMethod {
                        tag: ConstantTag::InterfaceMethod,
                        class: pool.address(self.bytes.try_get_u16()?),
                        name_and_type: pool.address(self.bytes.try_get_u16()?),
                    })
                }
                ConstantTag::String => ConstantEntry::String(ConstantString {
                    tag: ConstantTag::String,
                    string: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::Integer => ConstantEntry::Integer(ConstantInteger {
                    tag: ConstantTag::Integer,
                    bytes: self.bytes.try_get_u32()?,
                }),
                ConstantTag::Float => ConstantEntry::Float(ConstantFloat {
                    tag: ConstantTag::Float,
                    bytes: self.bytes.try_get_f32()?,
                }),
                ConstantTag::Long => ConstantEntry::Long(ConstantLong {
                    tag: ConstantTag::Long,
                    bytes: self.bytes.try_get_u64()?,
                }),
                ConstantTag::Double => ConstantEntry::Double(ConstantDouble {
                    tag: ConstantTag::Float,
                    bytes: self.bytes.try_get_f64()?,
                }),
                ConstantTag::NameAndType => ConstantEntry::NameAndType(ConstantNameAndType {
                    tag: ConstantTag::NameAndType,
                    name: pool.address(self.bytes.try_get_u16()?),
                    descriptor: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::Utf8 => {
                    let length = self.bytes.try_get_u16()?;
                    let mut bytes: Vec<u8> = Vec::with_capacity(length.into());

                    for _ in 0..length {
                        bytes.push(self.bytes.try_get_u8()?);
                    }

                    ConstantEntry::Utf8(ConstantUtf8 {
                        tag: ConstantTag::Utf8,
                        length,
                        bytes,
                    })
                }
                ConstantTag::MethodHandle => ConstantEntry::MethodHandle(ConstantMethodHandle {
                    tag: ConstantTag::MethodHandle,
                    kind: self.bytes.try_get_u8()?,
                    index: self.bytes.try_get_u16()?,
                }),
                ConstantTag::MethodType => ConstantEntry::MethodType(ConstantMethodType {
                    tag: ConstantTag::MethodType,
                    descriptor: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::Dynamic => ConstantEntry::Dynamic(ConstantDynamic {
                    tag: ConstantTag::Dynamic,
                    method_index: self.bytes.try_get_u16()?,
                    name_and_type: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::InvokeDynamic => ConstantEntry::InvokeDynamic(ConstantInvokeDynamic {
                    tag: ConstantTag::InvokeDynamic,
                    method_index: self.bytes.try_get_u16()?,
                    name_and_type: pool.address(self.bytes.try_get_u16()?),
                }),
                ConstantTag::Module => todo!(),
                ConstantTag::Package => todo!(),
            };

            let should_reserve_next =
                matches!(entry, ConstantEntry::Long(_) | ConstantEntry::Double(_));
            pool.insert(entry);

            // Special case: 64 Bit types are supposed to take up 2 slots
            // So, insert a dummy and increment the index by an additional 1
            if should_reserve_next {
                pool.insert(ConstantEntry::Reserved);
                i += 1;
            }

            i += 1;
        }

        Ok(pool)
    }

    fn parse_interfaces(&mut self, pool: &ConstantPool) -> Result<Interfaces> {
        let length = self.bytes.try_get_u16()?;
        let mut interfaces = Interfaces {
            values: Vec::with_capacity(length.into()),
        };

        for _ in 0..length {
            interfaces
                .values
                .push(pool.address(self.bytes.try_get_u16()?));
        }

        Ok(interfaces)
    }

    fn parse_fields(&mut self, pool: &ConstantPool) -> Result<Fields> {
        let length = self.bytes.try_get_u16()?;
        let mut fields = Fields {
            values: Vec::with_capacity(length.into()),
        };

        for _ in 0..length {
            fields.values.push(Field {
                flags: FieldAccessFlags::from_bits(self.bytes.try_get_u16()?)?,
                name: pool.address(self.bytes.try_get_u16()?),
                descriptor: pool.address(self.bytes.try_get_u16()?),
                attributes: Attributes::parse(&mut self.bytes, pool)?,
            });
        }

        Ok(fields)
    }

    fn parse_methods(&mut self, pool: &ConstantPool) -> Result<Methods> {
        let length = self.bytes.try_get_u16()?;
        let mut methods = Methods {
            values: Vec::with_capacity(length.into()),
        };

        for _ in 0..length {
            methods.values.push(Method {
                flags: MethodAccessFlags::from_bits(self.bytes.try_get_u16()?)?,
                name: pool.address(self.bytes.try_get_u16()?),
                descriptor: pool.address(self.bytes.try_get_u16()?),
                attributes: Attributes::parse(&mut self.bytes, pool)?,
            });
        }

        Ok(methods)
    }

    pub fn parse(&mut self) -> ParseResult {
        let magic = self.bytes.try_get_u32()?;

        // Format checking: The first four bytes must contain the right magic number
        if magic != MAGIC {
            return Err(anyhow!("invalid magic value '{}'", magic));
        }

        let minor = self.bytes.try_get_u16()?;
        let major = self.bytes.try_get_u16()?;

        let meta_data = MetaData {
            minor_version: minor,
            major_version: major,
        };

        let constant_pool = self.parse_constant_pool()?;
        // Format checking: The constant pool must satisfy the constraints documented throughout §4.4.
        // All field references and method references in the constant pool must have valid
        // names, valid classes, and valid descriptors (§4.3).
        constant_pool.perform_format_checking()?;

        let access_flags = ClassFileAccessFlags::from_bits(self.bytes.try_get_u16()?)?;
        let this_class: Addressed<ConstantClass> = constant_pool.address(self.bytes.try_get_u16()?);

        let super_class_index = self.bytes.try_get_u16()?;
        let mut super_class: Option<Addressed<ConstantClass>> = None;
        if super_class_index != 0 {
            super_class = Some(constant_pool.address(super_class_index));
        }

        let interfaces = self.parse_interfaces(&constant_pool)?;
        let fields = self.parse_fields(&constant_pool)?;
        let methods = self.parse_methods(&constant_pool)?;
        let attributes = Attributes::parse(&mut self.bytes, &constant_pool)?;

        // Format checking: The class file must not be truncated or have extra bytes at the end
        if !self.bytes.is_empty() {
            return Err(anyhow!("classfile has extra bytes at the end"));
        }

        Ok(ClassFile {
            constant_pool,
            meta_data,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }
}
