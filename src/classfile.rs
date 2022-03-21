use crate::types::classfile_type::{
    AttributeInfo, AttributeInfoEntry, ByteCode, ConstantPoolData, ConstantPoolEntry,
    ConstantPoolInfo, ConstantPoolTag, FieldInfo, FieldInfoEntry, Index, InterfaceInfo, MethodInfo,
    MethodInfoEntry,
};
use bytes::Buf;

pub const MAGIC: u32 = 0xCAFEBABE;
pub const MAX_SUPPORTED_MAJOR: u16 = 61;
pub const MAX_SUPPORTED_MINOR: u16 = 0;

pub fn invalid_format(file_name: &str, msg: &str) -> ! {
    eprintln!(
        "[ERROR] Invalid format for class file {} ({})",
        file_name, msg
    );
    std::process::exit(1)
}

fn make_utf8_string(byte_code: &mut ByteCode) -> ConstantPoolData {
    let length = byte_code.data().get_u16();

    let mut bytes: Vec<u8> = Vec::with_capacity(length as usize);
    let mut idx = 0;

    while idx < length {
        bytes.push(byte_code.data().get_u8());
        idx += 1;
    }

    ConstantPoolData::Utf8 {
        bytes: bytes.clone(),
        length,
        as_str: String::from_utf8(bytes).unwrap(),
    }
}

fn make_const_pool_data(byte_code: &mut ByteCode, tag: &ConstantPoolTag) -> ConstantPoolData {
    match tag {
        ConstantPoolTag::Utf8 => make_utf8_string(byte_code),
        ConstantPoolTag::Integer => ConstantPoolData::Integer {
            bytes: byte_code.data().get_u32(),
        },
        ConstantPoolTag::Float => ConstantPoolData::Float {
            bytes: byte_code.data().get_f32(),
        },
        ConstantPoolTag::Long => ConstantPoolData::Long {
            low_bytes: byte_code.data().get_u32(),
            high_bytes: byte_code.data().get_u32(),
        },
        ConstantPoolTag::Double => ConstantPoolData::Double {
            low_bytes: byte_code.data().get_f32(),
            high_bytes: byte_code.data().get_f32(),
        },
        ConstantPoolTag::Class => ConstantPoolData::Class {
            name_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::String => ConstantPoolData::String {
            utf8_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::FieldRef => ConstantPoolData::FieldRef {
            class_index: byte_code.data().get_u16(),
            name_and_type_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::MethodRef => ConstantPoolData::MethodRef {
            class_index: byte_code.data().get_u16(),
            name_and_type_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::InterfaceMethodRef => ConstantPoolData::InterfaceMethodRef {
            class_index: byte_code.data().get_u16(),
            name_and_type_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::NameAndType => ConstantPoolData::NameAndType {
            name_index: byte_code.data().get_u16(),
            descriptor_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::MethodHandle => ConstantPoolData::MethodHandle {
            reference_kind: byte_code.data().get_u8(),
            reference_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::MethodType => ConstantPoolData::MethodType {
            descriptor_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::Dynamic => ConstantPoolData::Dynamic {
            bootstrap_method_attr_index: byte_code.data().get_u16(),
            name_and_type_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::InvokeDynamic => ConstantPoolData::InvokeDynamic {
            bootstrap_method_attr_index: byte_code.data().get_u16(),
            name_and_type_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::Module => ConstantPoolData::Module {
            name_index: byte_code.data().get_u16(),
        },
        ConstantPoolTag::Package => ConstantPoolData::Package {
            name_index: byte_code.data().get_u16(),
        },
    }
}

fn make_const_pool(byte_code: &mut ByteCode, pool_size: u16) -> ConstantPoolInfo {
    let mut const_pool = ConstantPoolInfo::new(pool_size);

    //TODO: figure out what index 0 should have in the const pool and alter this

    // -1 because the const pool is indexed from 1 -> len - 1
    while const_pool.data().len() < (pool_size - 1) as usize {
        let tag = ConstantPoolTag::new(byte_code.data().get_u8(), byte_code);
        let data = make_const_pool_data(byte_code, &tag);
        let entry = ConstantPoolEntry::new(tag, data);

        const_pool.data().push(entry);
    }
    const_pool
}

fn make_interface_info(byte_code: &mut ByteCode, length: u16) -> InterfaceInfo {
    let mut out: Vec<u16> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        out.push(byte_code.data().get_u16());
    }
    InterfaceInfo::new(out)
}

fn make_attribute_info(byte_code: &mut ByteCode, length: u16) -> AttributeInfo {
    let mut out: Vec<AttributeInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        let attribute_name_index = byte_code.data().get_u16();
        let attribute_length = byte_code.data().get_u32();

        let mut attributes: Vec<u8> = Vec::with_capacity(attribute_length as usize);

        while attributes.len() < attribute_length as usize {
            attributes.push(byte_code.data().get_u8());
        }
        out.push(AttributeInfoEntry::new(
            attribute_name_index,
            attribute_length,
            attributes,
        ));
    }
    AttributeInfo::new(out)
}

fn make_field_info(byte_code: &mut ByteCode, length: u16) -> FieldInfo {
    let mut out: Vec<FieldInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        let access_flags = byte_code.data().get_u16();
        let name_index = byte_code.data().get_u16();
        let descriptor_index = byte_code.data().get_u16();
        let attributes_count = byte_code.data().get_u16();
        let attribute_info = make_attribute_info(byte_code, attributes_count);

        out.push(FieldInfoEntry::new(
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        ))
    }

    FieldInfo::new(out)
}

fn make_method_info(byte_code: &mut ByteCode, length: u16) -> MethodInfo {
    let mut out: Vec<MethodInfoEntry> = Vec::with_capacity(length as usize);
    while out.len() < length as usize {
        let access_flags = byte_code.data().get_u16();
        let name_index = byte_code.data().get_u16();
        let descriptor_index = byte_code.data().get_u16();
        let attributes_count = byte_code.data().get_u16();
        let attribute_info = make_attribute_info(byte_code, attributes_count);

        out.push(MethodInfoEntry::new(
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        ))
    }

    MethodInfo::new(out)
}

pub struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    const_pool_count: u16,
    const_pool_info: ConstantPoolInfo,
    access_flags: u16,
    this_class: Index,
    super_class: Index,
    interface_count: u16,
    interface_info: InterfaceInfo,
    field_count: u16,
    field_info: FieldInfo,
    method_count: u16,
    method_info: MethodInfo,
    attribute_count: u16,
    attribute_info: AttributeInfo,
}

impl ClassFile {
    pub fn new(byte_code: &mut ByteCode, class_file: &str) -> Self {
        let magic = byte_code.data().get_u32();

        if magic != MAGIC {
            invalid_format(class_file, "magic value not present")
        }

        let minor = byte_code.data().get_u16();

        if minor > MAX_SUPPORTED_MINOR {
            invalid_format(class_file, "minor version not supported")
        }

        let major = byte_code.data().get_u16();

        if major > MAX_SUPPORTED_MAJOR {
            invalid_format(class_file, "major version not supported")
        }

        let const_pool_count = byte_code.data().get_u16();

        let const_pool_info = make_const_pool(byte_code, const_pool_count);
        let access_flags = byte_code.data().get_u16();
        let this_class = byte_code.data().get_u16();
        let super_class = byte_code.data().get_u16();
        let interface_count = byte_code.data().get_u16();
        let interface_info = make_interface_info(byte_code, interface_count);
        let field_count = byte_code.data().get_u16();
        let field_info = make_field_info(byte_code, field_count);
        let method_count = byte_code.data().get_u16();
        let method_info = make_method_info(byte_code, method_count);
        let attribute_count = byte_code.data().get_u16();
        let attribute_info = make_attribute_info(byte_code, attribute_count);

        Self {
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
        }
    }

    pub fn validate(self) -> Result<ClassFile, &'static str> {
        Ok(self)
    }
}
