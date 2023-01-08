use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};
use support::bytes_ext::SafeBuf;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum Opcode {
    // Constants Loads Stores
    NOP,
    ACONST_NULL,
    ICONST_M1,
    ICONST_0,
    ICONST_1,
    ICONST_2,
    ICONST_3,
    ICONST_4,
    ICONST_5,
    LCONST_0,
    LCONST_1,
    FCONST_0,
    FCONST_1,
    FCONST_2,
    DCONST_0,
    DCONST_1,
    BIPUSH(i8),
    SIPUSH(i16),
    LDC(u8),
    LDC_W(u16),
    LDC2_W(u16),
    ILOAD(u8),
    LLOAD(u8),
    FLOAD(u8),
    DLOAD,
    ALOAD(u8),
    ILOAD_0,
    ILOAD_1,
    ILOAD_2,
    ILOAD_3,
    LLOAD_0,
    LLOAD_1,
    LLOAD_2,
    LLOAD_3,
    FLOAD_0,
    FLOAD_1,
    FLOAD_2,
    FLOAD_3,
    DLOAD_0,
    DLOAD_1,
    DLOAD_2,
    DLOAD_3,
    ALOAD_0,
    ALOAD_1,
    ALOAD_2,
    ALOAD_3,
    IALOAD,
    LALOAD,
    FALOAD,
    DALOAD,
    AALOAD,
    BALOAD,
    CALOAD,
    SALOAD,
    ISTORE(u8),
    LSTORE(u8),
    FSTORE(u8),
    DSTORE,
    ASTORE(u8),
    ISTORE_0,
    ISTORE_1,
    ISTORE_2,
    ISTORE_3,
    LSTORE_0,
    LSTORE_1,
    LSTORE_2,
    LSTORE_3,
    FSTORE_0,
    FSTORE_1,
    FSTORE_2,
    FSTORE_3,
    DSTORE_0,
    DSTORE_1,
    DSTORE_2,
    DSTORE_3,
    ASTORE_0,
    ASTORE_1,
    ASTORE_2,
    ASTORE_3,
    IASTORE,
    LASTORE,
    FASTORE,
    DASTORE,
    AASTORE,
    BASTORE,
    CASTORE,
    SASTORE,

    // Stack Math Conversions
    POP,
    POP2,
    DUP,
    DUP_X1,
    DUP_X2,
    DUP2,
    DUP2_X1,
    DUP2_X2,
    SWAP,
    IADD,
    LADD,
    FADD,
    DADD,
    ISUB,
    LSUB,
    FSUB,
    DSUB,
    IMUL,
    LMUL,
    FMUL,
    DMUL,
    IDIV,
    LDIV,
    FDIV,
    DDIV,
    IREM,
    LREM,
    FREM,
    DREM,
    INEG,
    LNEG,
    FNEG,
    DNEG,
    ISHL,
    LSHL,
    ISHR,
    LSHR,
    IUSHR,
    LUSHR,
    IAND,
    LAND,
    IOR,
    LOR,
    IXOR,
    LXOR,
    IINC(u8, i8),
    I2L,
    I2F,
    I2D,
    L2I,
    L2F,
    L2D,
    F2I,
    F2L,
    F2D,
    D2I,
    D2L,
    D2F,
    I2B,
    I2C,
    I2S,

    // Comparisons
    LCMP,
    FCMPL,
    FCMPG,
    DCMPL,
    DCMPG,
    IFEQ(i16),
    IFNE(i16),
    IFLT(i16),
    IFGE(i16),
    IFGT(i16),
    IFLE(i16),
    IF_ICMPEQ(i16),
    IF_ICMPNE(i16),
    IF_ICMPLT(i16),
    IF_ICMPGE(i16),
    IF_ICMPGT(i16),
    IF_ICMPLE(i16),
    IF_ACMPEQ(i16),
    IF_ACMPNE(i16),

    // Control
    GOTO(i16),
    JSR,
    RET,
    TABLESWITCH(TableSwitch),
    LOOKUPSWITCH,
    IRETURN,
    LRETURN,
    FRETURN,
    DRETURN,
    ARETURN,
    RETURN,

    // References
    GETSTATIC(u16),
    PUTSTATIC(u16),
    GETFIELD(u16),
    PUTFIELD(u16),
    INVOKEVIRTUAL(u16),
    INVOKESPECIAL(u16),
    INVOKESTATIC(u16),
    INVOKEINTERFACE(u16, u8, u8),
    INVOKEDYNAMIC,
    NEW(u16),
    NEWARRAY(u8),
    ANEWARRAY(u16),
    ARRAYLENGTH,
    ATHROW,
    CHECKCAST(u16),
    INSTANCEOF(u16),
    MONITORENTER,
    MONITOREXIT,

    // Extended
    WIDE,
    MULTIANEWARRAY,
    IFNULL(i16),
    IFNONNULL(i16),
    GOTO_W,
    JSR_W,

    // Reserved
    BREAKPOINT,
    IMPDEP1,
    IMPDEP2,
}

#[derive(Debug, Clone)]
pub struct TableSwitch {
    pub default: u32,
    pub low: u32,
    pub high: u32,
    pub pairs: Vec<(u32, u32)>,
}

impl Opcode {
    pub fn decode(bytes: &mut BytesMut, pc: i32) -> Result<Opcode> {
        let instruction = bytes.try_get_u8()?;
        Ok(match instruction {
            // Constants Loads Stores
            0x00 => Opcode::NOP,
            0x01 => Opcode::ACONST_NULL,
            0x02 => Opcode::ICONST_M1,
            0x03 => Opcode::ICONST_0,
            0x04 => Opcode::ICONST_1,
            0x05 => Opcode::ICONST_2,
            0x06 => Opcode::ICONST_3,
            0x07 => Opcode::ICONST_4,
            0x08 => Opcode::ICONST_5,
            0x09 => Opcode::LCONST_0,
            0x0a => Opcode::LCONST_1,
            0x0b => Opcode::FCONST_0,
            0x0c => Opcode::FCONST_1,
            0x0d => Opcode::FCONST_2,
            0x0e => Opcode::DCONST_0,
            0x0f => Opcode::DCONST_1,
            0x10 => Opcode::BIPUSH(bytes.try_get_i8()?),
            0x11 => Opcode::SIPUSH(bytes.try_get_i16()?),
            0x12 => Opcode::LDC(bytes.try_get_u8()?),
            0x13 => Opcode::LDC_W(bytes.try_get_u16()?),
            0x14 => Opcode::LDC2_W(bytes.try_get_u16()?),
            0x15 => Opcode::ILOAD(bytes.try_get_u8()?),
            0x16 => Opcode::LLOAD(bytes.try_get_u8()?),
            0x17 => Opcode::FLOAD(bytes.try_get_u8()?),
            0x18 => Opcode::DLOAD,
            0x19 => Opcode::ALOAD(bytes.try_get_u8()?),
            0x1a => Opcode::ILOAD_0,
            0x1b => Opcode::ILOAD_1,
            0x1c => Opcode::ILOAD_2,
            0x1d => Opcode::ILOAD_3,
            0x1e => Opcode::LLOAD_0,
            0x1f => Opcode::LLOAD_1,
            0x20 => Opcode::LLOAD_2,
            0x21 => Opcode::LLOAD_3,
            0x22 => Opcode::FLOAD_0,
            0x23 => Opcode::FLOAD_1,
            0x24 => Opcode::FLOAD_2,
            0x25 => Opcode::FLOAD_3,
            0x26 => Opcode::DLOAD_0,
            0x27 => Opcode::DLOAD_1,
            0x28 => Opcode::DLOAD_2,
            0x29 => Opcode::DLOAD_3,
            0x2a => Opcode::ALOAD_0,
            0x2b => Opcode::ALOAD_1,
            0x2c => Opcode::ALOAD_2,
            0x2d => Opcode::ALOAD_3,
            0x2e => Opcode::IALOAD,
            0x2f => Opcode::LALOAD,
            0x30 => Opcode::FALOAD,
            0x31 => Opcode::DALOAD,
            0x32 => Opcode::AALOAD,
            0x33 => Opcode::BALOAD,
            0x34 => Opcode::CALOAD,
            0x35 => Opcode::SALOAD,
            0x36 => Opcode::ISTORE(bytes.try_get_u8()?),
            0x37 => Opcode::LSTORE(bytes.try_get_u8()?),
            0x38 => Opcode::FSTORE(bytes.try_get_u8()?),
            0x39 => Opcode::DSTORE,
            0x3a => Opcode::ASTORE(bytes.try_get_u8()?),
            0x3b => Opcode::ISTORE_0,
            0x3c => Opcode::ISTORE_1,
            0x3d => Opcode::ISTORE_2,
            0x3e => Opcode::ISTORE_3,
            0x3f => Opcode::LSTORE_0,
            0x40 => Opcode::LSTORE_1,
            0x41 => Opcode::LSTORE_2,
            0x42 => Opcode::LSTORE_3,
            0x43 => Opcode::FSTORE_0,
            0x44 => Opcode::FSTORE_1,
            0x45 => Opcode::FSTORE_2,
            0x46 => Opcode::FSTORE_3,
            0x47 => Opcode::DSTORE_0,
            0x48 => Opcode::DSTORE_1,
            0x49 => Opcode::DSTORE_2,
            0x4a => Opcode::DSTORE_3,
            0x4b => Opcode::ASTORE_0,
            0x4c => Opcode::ASTORE_1,
            0x4d => Opcode::ASTORE_2,
            0x4e => Opcode::ASTORE_3,
            0x4f => Opcode::IASTORE,
            0x50 => Opcode::LASTORE,
            0x51 => Opcode::FASTORE,
            0x52 => Opcode::DASTORE,
            0x53 => Opcode::AASTORE,
            0x54 => Opcode::BASTORE,
            0x55 => Opcode::CASTORE,
            0x56 => Opcode::SASTORE,

            // Stack Math Conversions
            0x57 => Opcode::POP,
            0x58 => Opcode::POP2,
            0x59 => Opcode::DUP,
            0x5a => Opcode::DUP_X1,
            0x5b => Opcode::DUP_X2,
            0x5c => Opcode::DUP2,
            0x5d => Opcode::DUP2_X1,
            0x5e => Opcode::DUP2_X2,
            0x5f => Opcode::SWAP,
            0x60 => Opcode::IADD,
            0x61 => Opcode::LADD,
            0x62 => Opcode::FADD,
            0x63 => Opcode::DADD,
            0x64 => Opcode::ISUB,
            0x65 => Opcode::LSUB,
            0x66 => Opcode::FSUB,
            0x67 => Opcode::DSUB,
            0x68 => Opcode::IMUL,
            0x69 => Opcode::LMUL,
            0x6a => Opcode::FMUL,
            0x6b => Opcode::DMUL,
            0x6c => Opcode::IDIV,
            0x6d => Opcode::LDIV,
            0x6e => Opcode::FDIV,
            0x6f => Opcode::DDIV,
            0x70 => Opcode::IREM,
            0x71 => Opcode::LREM,
            0x72 => Opcode::FREM,
            0x73 => Opcode::DREM,
            0x74 => Opcode::INEG,
            0x75 => Opcode::LNEG,
            0x76 => Opcode::FNEG,
            0x77 => Opcode::DNEG,
            0x78 => Opcode::ISHL,
            0x79 => Opcode::LSHL,
            0x7a => Opcode::ISHR,
            0x7b => Opcode::LSHR,
            0x7c => Opcode::IUSHR,
            0x7d => Opcode::LUSHR,
            0x7e => Opcode::IAND,
            0x7f => Opcode::LAND,
            0x80 => Opcode::IOR,
            0x81 => Opcode::LOR,
            0x82 => Opcode::IXOR,
            0x83 => Opcode::LXOR,
            0x84 => Opcode::IINC(bytes.try_get_u8()?, bytes.try_get_i8()?),
            0x85 => Opcode::I2L,
            0x86 => Opcode::I2F,
            0x87 => Opcode::I2D,
            0x88 => Opcode::L2I,
            0x89 => Opcode::L2F,
            0x8a => Opcode::L2D,
            0x8b => Opcode::F2I,
            0x8c => Opcode::F2L,
            0x8d => Opcode::F2D,
            0x8e => Opcode::D2I,
            0x8f => Opcode::D2L,
            0x90 => Opcode::D2F,
            0x91 => Opcode::I2B,
            0x92 => Opcode::I2C,
            0x93 => Opcode::I2S,

            // Comparisons
            0x94 => Opcode::LCMP,
            0x95 => Opcode::FCMPL,
            0x96 => Opcode::FCMPG,
            0x97 => Opcode::DCMPL,
            0x98 => Opcode::DCMPG,
            0x99 => Opcode::IFEQ(bytes.try_get_i16()?),
            0x9a => Opcode::IFNE(bytes.try_get_i16()?),
            0x9b => Opcode::IFLT(bytes.try_get_i16()?),
            0x9c => Opcode::IFGE(bytes.try_get_i16()?),
            0x9d => Opcode::IFGT(bytes.try_get_i16()?),
            0x9e => Opcode::IFLE(bytes.try_get_i16()?),
            0x9f => Opcode::IF_ICMPEQ(bytes.try_get_i16()?),
            0xa0 => Opcode::IF_ICMPNE(bytes.try_get_i16()?),
            0xa1 => Opcode::IF_ICMPLT(bytes.try_get_i16()?),
            0xa2 => Opcode::IF_ICMPGE(bytes.try_get_i16()?),
            0xa3 => Opcode::IF_ICMPGT(bytes.try_get_i16()?),
            0xa4 => Opcode::IF_ICMPLE(bytes.try_get_i16()?),
            0xa5 => Opcode::IF_ACMPEQ(bytes.try_get_i16()?),
            0xa6 => Opcode::IF_ACMPNE(bytes.try_get_i16()?),

            // Control
            0xa7 => Opcode::GOTO(bytes.try_get_i16()?),
            0xa8 => Opcode::JSR,
            0xa9 => Opcode::RET,
            0xaa => {
                let pad_len = 3 - (pc % 4);
                bytes.advance(pad_len as usize);

                let default = bytes.try_get_u32()?;

                let low = bytes.try_get_u32()?;
                let high = bytes.try_get_u32()?;

                let mut idx = low;

                let len = (high - low + 1) as usize;
                let mut pairs: Vec<(u32, u32)> = Vec::with_capacity(len);
                for _ in 0..len {
                    pairs.push((idx, bytes.try_get_u32()?));
                    idx += 1;
                }

                Opcode::TABLESWITCH(TableSwitch {
                    default,
                    low,
                    high,
                    pairs,
                })
            }
            0xab => Opcode::LOOKUPSWITCH,
            0xac => Opcode::IRETURN,
            0xad => Opcode::LRETURN,
            0xae => Opcode::FRETURN,
            0xaf => Opcode::DRETURN,
            0xb0 => Opcode::ARETURN,
            0xb1 => Opcode::RETURN,

            // References
            0xb2 => Opcode::GETSTATIC(bytes.try_get_u16()?),
            0xb3 => Opcode::PUTSTATIC(bytes.try_get_u16()?),
            0xb4 => Opcode::GETFIELD(bytes.try_get_u16()?),
            0xb5 => Opcode::PUTFIELD(bytes.try_get_u16()?),
            0xb6 => Opcode::INVOKEVIRTUAL(bytes.try_get_u16()?),
            0xb7 => Opcode::INVOKESPECIAL(bytes.try_get_u16()?),
            0xb8 => Opcode::INVOKESTATIC(bytes.try_get_u16()?),
            0xb9 => Opcode::INVOKEINTERFACE(
                bytes.try_get_u16()?,
                bytes.try_get_u8()?,
                bytes.try_get_u8()?,
            ),
            0xba => Opcode::INVOKEDYNAMIC,
            0xbb => Opcode::NEW(bytes.try_get_u16()?),
            0xbc => Opcode::NEWARRAY(bytes.try_get_u8()?),
            0xbd => Opcode::ANEWARRAY(bytes.try_get_u16()?),
            0xbe => Opcode::ARRAYLENGTH,
            0xbf => Opcode::ATHROW,
            0xc0 => Opcode::CHECKCAST(bytes.try_get_u16()?),
            0xc1 => Opcode::INSTANCEOF(bytes.try_get_u16()?),
            0xc2 => Opcode::MONITORENTER,
            0xc3 => Opcode::MONITOREXIT,

            // Extended
            0xc4 => Opcode::WIDE,
            0xc5 => Opcode::MULTIANEWARRAY,
            0xc6 => Opcode::IFNULL(bytes.try_get_i16()?),
            0xc7 => Opcode::IFNONNULL(bytes.try_get_i16()?),
            0xc8 => Opcode::GOTO_W,
            0xc9 => Opcode::JSR_W,
            // Reserved
            0xca => Opcode::BREAKPOINT,
            0xfe => Opcode::IMPDEP1,
            0xff => Opcode::IMPDEP2,
            e => return Err(anyhow!("unknown opcode {:#01x}", e)),
        })
    }
}
