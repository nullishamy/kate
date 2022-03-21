use anyhow::{anyhow, Result};
use bitflags::bitflags;

bitflags! {
    pub struct ClassFileAccessFlag: u32 {
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

pub struct ClassFileAccessFlags {
    flags: ClassFileAccessFlag,
}

impl ClassFileAccessFlags {
    pub fn from_bits(raw: u16) -> Result<Self> {
        let flags = ClassFileAccessFlag::from_bits(raw as u32);

        if flags.is_none() {
            return Err(anyhow!("invalid class access flags {:b}", raw));
        }
        Ok(Self {
            flags: flags.unwrap(),
        })
    }

    pub fn has(&self, other: ClassFileAccessFlag) -> bool {
        self.flags.contains(other)
    }
}
