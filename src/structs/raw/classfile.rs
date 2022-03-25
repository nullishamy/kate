use crate::structs::raw::attribute::AttributeEntry;
use crate::structs::raw::constant_pool::PoolEntry;
use crate::structs::raw::field::FieldEntry;
use crate::structs::raw::method::MethodEntry;

pub const MAGIC: u32 = 0xCAFEBABE;
pub const MAX_SUPPORTED_MAJOR: u16 = 61;
pub const MAX_SUPPORTED_MINOR: u16 = 0;

pub struct RawClassFile {
    pub magic: u32,

    pub minor_version: u16,
    pub major_version: u16,

    pub const_pool_count: u16,
    pub const_pool_info: Vec<Option<PoolEntry>>,

    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,

    pub interface_count: u16,
    pub interface_info: Vec<u16>,

    pub field_count: u16,
    pub field_info: Vec<FieldEntry>,

    pub method_count: u16,
    pub method_info: Vec<MethodEntry>,

    pub attribute_count: u16,
    pub attribute_info: Vec<AttributeEntry>,
}
