//! defines the bitflags used in class file parsing
//! these flags are used to determine access levels and features of various entities
//! this module defines a private macro to generate flag implementations based on a flag struct

use anyhow::Result;
use bitflags::bitflags;
use tracing::warn;

macro_rules! impl_flags {
    ( $flag_type:ident, $impl_type:ident ) => {
        #[derive(Clone, Debug)]
        pub struct $impl_type {
            pub flags: $flag_type,
        }

        impl $impl_type {
            pub fn from_bits(raw: u16) -> Result<Self> {
                let mut flags = <$flag_type>::from_bits(raw);

                if flags.is_none() {
                    warn!("unrecognised bits {:b} for {}", raw, stringify!($flag_type));
                    flags = Some(<$flag_type>::from_bits_truncate(raw));
                }

                Ok(Self {
                    flags: flags.unwrap(),
                })
            }

            pub fn has(&self, other: $flag_type) -> bool {
                self.flags.contains(other)
            }
        }
    };
}

bitflags! {
    pub struct ClassFileAccessFlag: u16 {
         const PUBLIC = 0x0001;
         const FINAL = 0x0010;
         const SUPER = 0x0020;
         const INTERFACE = 0x0200;
         const ABSTRACT = 0x0400;
         const SYNTHETIC = 0x1000;
         const ANNOTATION = 0x2000;
         const ENUM = 0x4000;
         const MODULE = 0x8000;
    }
}

bitflags! {
    pub struct MethodAccessFlag: u16 {
         const PUBLIC = 0x0001;
         const PRIVATE = 0x0002;
         const PROTECTED = 0x0004;
         const STATIC = 0x0008;
         const FINAL = 0x0010;
         const SYNCHRONIZED = 0x0020;
         const BRIDGE = 0x0040;
         const VARARGS = 0x0080;
         const NATIVE = 0x0100;
         const ABSTRACT = 0x0400;
         const STRICT_FP = 0x0800;
         const SYNTHETIC = 0x1000;
    }
}

bitflags! {
    pub struct FieldAccessFlag: u16 {
         const PUBLIC = 0x0001;
         const PRIVATE = 0x0002;
         const PROTECTED = 0x0004;
         const STATIC = 0x0008;
         const FINAL = 0x0010;
         const VOLATILE = 0x0040;
         const SYNTHETIC = 0x1000;
         const ENUM = 0x4000;
    }
}

impl_flags!(MethodAccessFlag, MethodAccessFlags);
impl_flags!(ClassFileAccessFlag, ClassFileAccessFlags);
impl_flags!(FieldAccessFlag, FieldAccessFlags);
