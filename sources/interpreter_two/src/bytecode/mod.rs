use std::fmt;

use crate::{object::RuntimeValue, Context, VM};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use support::bytes_ext::SafeBuf;
mod ops;

pub trait Instruction: fmt::Debug {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        Ok(ctx.pc)
    }
}

/// Utility to box a value. Used below to box each instruction that we decode
fn b<T>(v: T) -> Box<T> {
    Box::new(v)
}

pub fn decode_instruction(_vm: &VM, bytes: &mut BytesMut) -> Result<Box<dyn Instruction>> {
    let instruction = bytes.try_get_u8()?;
    Ok(match instruction {
        0x00 => b(ops::Nop {}),

        // Constants Loads Stores
        //  0x01 => Opcode::ACONST_NULL,
         0x02 => b(ops::PushConst {
            value: RuntimeValue::Integral((-1_i8).into()),
        }),
        0x03 => b(ops::PushConst {
            value: RuntimeValue::Integral(0.into()),
        }),
        //  0x04 => Opcode::ICONST_1,
        //  0x05 => Opcode::ICONST_2,
        0x06 => b(ops::PushConst {
            value: RuntimeValue::Integral((3_i32).into()),
        }),
        //  0x07 => Opcode::ICONST_4,
        0x08 => b(ops::PushConst {
            value: RuntimeValue::Integral((5_i32).into()),
        }),
        0x09 => b(ops::PushConst {
            value: RuntimeValue::Integral((0_i64).into()),
        }),
        0x0a => b(ops::PushConst {
            value: RuntimeValue::Integral((1_i64).into()),
        }),
        //  0x0b => Opcode::FCONST_0,
        //  0x0c => Opcode::FCONST_1,
        0x0d => b(ops::PushConst {
            value: RuntimeValue::Floating((2.0_f32).into()),
        }),
        0x0e => b(ops::PushConst {
            value: RuntimeValue::Floating((0.0_f64).into()),
        }),
        0x0f => b(ops::PushConst {
            value: RuntimeValue::Floating((1.0_f64).into()),
        }),
        0x10 => b(ops::PushConst {
            value: RuntimeValue::Integral(bytes.try_get_i8()?.into()),
        }),
        //  0x11 => Opcode::SIPUSH(bytes.try_get_i16()?),
        //  0x12 => Opcode::LDC(bytes.try_get_u8()?),
        //  0x13 => Opcode::LDC_W(bytes.try_get_u16()?),
        0x14 => b(ops::Ldc2W {
            index: bytes.try_get_u16()?
        }),
        0x15 => b(ops::LoadLocal { index: bytes.try_get_u8()? as usize }),
        0x16 => b(ops::LoadLocal { index: bytes.try_get_u8()? as usize }),
        0x17 => b(ops::LoadLocal { index: bytes.try_get_u8()? as usize }),
        //  0x18 => Opcode::DLOAD,
        //  0x19 => Opcode::ALOAD(bytes.try_get_u8()?),
        //  0x1a => Opcode::ILOAD_0,
        0x1b => b(ops::LoadLocal { index: 1 }),
        0x1c => b(ops::LoadLocal { index: 2 }),
        //  0x1d => Opcode::ILOAD_3,
        //  0x1e => Opcode::LLOAD_0,
        0x1f => b(ops::LoadLocal { index: 1 }),
        0x20 => b(ops::LoadLocal { index: 2 }),
        0x21 => b(ops::LoadLocal { index: 3 }),
        //  0x22 => Opcode::FLOAD_0,
        //  0x23 => Opcode::FLOAD_1,
        //  0x24 => Opcode::FLOAD_2,
        //  0x25 => Opcode::FLOAD_3,
        //  0x26 => Opcode::DLOAD_0,
        //  0x27 => Opcode::DLOAD_1,
        //  0x28 => Opcode::DLOAD_2,
        0x29 => b(ops::LoadLocal { index: 3 }),
        //  0x2a => Opcode::ALOAD_0,
        //  0x2b => Opcode::ALOAD_1,
        //  0x2c => Opcode::ALOAD_2,
        //  0x2d => Opcode::ALOAD_3,
        //  0x2e => Opcode::IALOAD,
        //  0x2f => Opcode::LALOAD,
        //  0x30 => Opcode::FALOAD,
        //  0x31 => Opcode::DALOAD,
        //  0x32 => Opcode::AALOAD,
        //  0x33 => Opcode::BALOAD,
        //  0x34 => Opcode::CALOAD,
        //  0x35 => Opcode::SALOAD,
        0x36 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        0x37 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        0x38 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        //  0x39 => Opcode::DSTORE,
        //  0x3a => Opcode::ASTORE(bytes.try_get_u8()?),
        //  0x3b => Opcode::ISTORE_0,
        0x3c => b(ops::StoreLocal { index: 1 }),
        0x3d => b(ops::StoreLocal { index: 2 }),
        //  0x3e => Opcode::ISTORE_3,
        //  0x3f => Opcode::LSTORE_0,
        0x40 => b(ops::StoreLocal { index: 1 }),
        0x41 => b(ops::StoreLocal { index: 2 }),
        0x42 => b(ops::StoreLocal { index: 3 }),
        //  0x44 => Opcode::FSTORE_1,
        //  0x45 => Opcode::FSTORE_2,
        //  0x46 => Opcode::FSTORE_3,
        //  0x47 => Opcode::DSTORE_0,
        //  0x48 => Opcode::DSTORE_1,
        //  0x49 => Opcode::DSTORE_2,
        0x4a => b(ops::StoreLocal { index: 3 }),
        0x4b => b(ops::StoreLocal { index: 0 }),
        //  0x4c => Opcode::ASTORE_1,
        //  0x4d => Opcode::ASTORE_2,
        //  0x4e => Opcode::ASTORE_3,
        //  0x4f => Opcode::IASTORE,
        //  0x50 => Opcode::LASTORE,
        //  0x51 => Opcode::FASTORE,
        //  0x52 => Opcode::DASTORE,
        //  0x53 => Opcode::AASTORE,
        //  0x54 => Opcode::BASTORE,
        //  0x55 => Opcode::CASTORE,
        //  0x56 => Opcode::SASTORE,

        //  // Stack Math Conversions
        //  0x57 => Opcode::POP,
        //  0x58 => Opcode::POP2,
        //  0x59 => Opcode::DUP,
        //  0x5a => Opcode::DUP_X1,
        //  0x5b => Opcode::DUP_X2,
        //  0x5c => Opcode::DUP2,
        //  0x5d => Opcode::DUP2_X1,
        //  0x5e => Opcode::DUP2_X2,
        //  0x5f => Opcode::SWAP,
        //  0x60 => Opcode::IADD,
        //  0x61 => Opcode::LADD,
        //  0x62 => Opcode::FADD,
        //  0x63 => Opcode::DADD,
        //  0x64 => Opcode::ISUB,
        //  0x65 => Opcode::LSUB,
        //  0x66 => Opcode::FSUB,
        //  0x67 => Opcode::DSUB,
        //  0x68 => Opcode::IMUL,
        //  0x69 => Opcode::LMUL,
        //  0x6a => Opcode::FMUL,
        //  0x6b => Opcode::DMUL,
        //  0x6c => Opcode::IDIV,
        //  0x6d => Opcode::LDIV,
        //  0x6e => Opcode::FDIV,
        //  0x6f => Opcode::DDIV,
        //  0x70 => Opcode::IREM,
        //  0x71 => Opcode::LREM,
        //  0x72 => Opcode::FREM,
        //  0x73 => Opcode::DREM,
        //  0x74 => Opcode::INEG,
        //  0x75 => Opcode::LNEG,
        //  0x76 => Opcode::FNEG,
        //  0x77 => Opcode::DNEG,
        //  0x78 => Opcode::ISHL,
        //  0x79 => Opcode::LSHL,
        //  0x7a => Opcode::ISHR,
        //  0x7b => Opcode::LSHR,
        //  0x7c => Opcode::IUSHR,
        //  0x7d => Opcode::LUSHR,
        //  0x7e => Opcode::IAND,
        //  0x7f => Opcode::LAND,
        //  0x80 => Opcode::IOR,
        //  0x81 => Opcode::LOR,
        //  0x82 => Opcode::IXOR,
        //  0x83 => Opcode::LXOR,
        //  0x84 => Opcode::IINC(bytes.try_get_u8()?, bytes.try_get_i8()?),
        //  0x85 => Opcode::I2L,
        //  0x86 => Opcode::I2F,
        //  0x87 => Opcode::I2D,
        //  0x88 => Opcode::L2I,
        //  0x89 => Opcode::L2F,
        //  0x8a => Opcode::L2D,
        //  0x8b => Opcode::F2I,
        //  0x8c => Opcode::F2L,
        //  0x8d => Opcode::F2D,
        //  0x8e => Opcode::D2I,
        //  0x8f => Opcode::D2L,
        //  0x90 => Opcode::D2F,
        //  0x91 => Opcode::I2B,
        //  0x92 => Opcode::I2C,
        //  0x93 => Opcode::I2S,

        //  // Comparisons
        //  0x94 => Opcode::LCMP,
        //  0x95 => Opcode::FCMPL,
        //  0x96 => Opcode::FCMPG,
        //  0x97 => Opcode::DCMPL,
        //  0x98 => Opcode::DCMPG,
        //  0x99 => Opcode::IFEQ(bytes.try_get_i16()?),
        //  0x9a => Opcode::IFNE(bytes.try_get_i16()?),
        //  0x9b => Opcode::IFLT(bytes.try_get_i16()?),
        //  0x9c => Opcode::IFGE(bytes.try_get_i16()?),
        //  0x9d => Opcode::IFGT(bytes.try_get_i16()?),
        //  0x9e => Opcode::IFLE(bytes.try_get_i16()?),
        //  0x9f => Opcode::IF_ICMPEQ(bytes.try_get_i16()?),
        //  0xa0 => Opcode::IF_ICMPNE(bytes.try_get_i16()?),
        //  0xa1 => Opcode::IF_ICMPLT(bytes.try_get_i16()?),
        //  0xa2 => Opcode::IF_ICMPGE(bytes.try_get_i16()?),
        //  0xa3 => Opcode::IF_ICMPGT(bytes.try_get_i16()?),
        //  0xa4 => Opcode::IF_ICMPLE(bytes.try_get_i16()?),
        //  0xa5 => Opcode::IF_ACMPEQ(bytes.try_get_i16()?),
        //  0xa6 => Opcode::IF_ACMPNE(bytes.try_get_i16()?),

        //  // Control
        //  0xa7 => Opcode::GOTO(bytes.try_get_i16()?),
        //  0xa8 => Opcode::JSR,
        //  0xa9 => Opcode::RET,
        //  0xaa => {
        //      let pad_len = 3 - (pc % 4);
        //      bytes.advance(pad_len as usize);

        //      let default = bytes.try_get_u32()?;

        //      let low = bytes.try_get_u32()?;
        //      let high = bytes.try_get_u32()?;

        //      let mut idx = low;

        //      let len = (high - low + 1) as usize;
        //      let mut pairs: Vec<(u32, u32)> = Vec::with_capacity(len);
        //      for _ in 0..len {
        //          pairs.push((idx, bytes.try_get_u32()?));
        //          idx += 1;
        //      }

        //      Opcode::TABLESWITCH(TableSwitch {
        //          default,
        //          low,
        //          high,
        //          pairs,
        //      })
        //  }
        //  0xab => Opcode::LOOKUPSWITCH,
        //  0xac => Opcode::IRETURN,
        //  0xad => Opcode::LRETURN,
        //  0xae => Opcode::FRETURN,
        //  0xaf => Opcode::DRETURN,
        //  0xb0 => Opcode::ARETURN,
        0xb1 => b(ops::Return {}),

        //  // References
        //  0xb2 => Opcode::GETSTATIC(bytes.try_get_u16()?),
        //  0xb3 => Opcode::PUTSTATIC(bytes.try_get_u16()?),
        //  0xb4 => Opcode::GETFIELD(bytes.try_get_u16()?),
        //  0xb5 => Opcode::PUTFIELD(bytes.try_get_u16()?),
        //  0xb6 => Opcode::INVOKEVIRTUAL(bytes.try_get_u16()?),
        //  0xb7 => Opcode::INVOKESPECIAL(bytes.try_get_u16()?),
        0xb8 => b(ops::InvokeStatic {
            index: bytes.try_get_u16()?,
        }),
        //  0xb9 => Opcode::INVOKEINTERFACE(
        //      bytes.try_get_u16()?,
        //      bytes.try_get_u8()?,
        //      bytes.try_get_u8()?,
        //  ),
        //  0xba => Opcode::INVOKEDYNAMIC,
        //  0xbb => Opcode::NEW(bytes.try_get_u16()?),
        //  0xbc => Opcode::NEWARRAY(bytes.try_get_u8()?),
        //  0xbd => Opcode::ANEWARRAY(bytes.try_get_u16()?),
        //  0xbe => Opcode::ARRAYLENGTH,
        //  0xbf => Opcode::ATHROW,
        //  0xc0 => Opcode::CHECKCAST(bytes.try_get_u16()?),
        //  0xc1 => Opcode::INSTANCEOF(bytes.try_get_u16()?),
        //  0xc2 => Opcode::MONITORENTER,
        //  0xc3 => Opcode::MONITOREXIT,

        //  // Extended
        //  0xc4 => Opcode::WIDE,
        //  0xc5 => Opcode::MULTIANEWARRAY,
        //  0xc6 => Opcode::IFNULL(bytes.try_get_i16()?),
        //  0xc7 => Opcode::IFNONNULL(bytes.try_get_i16()?),
        //  0xc8 => Opcode::GOTO_W,
        //  0xc9 => Opcode::JSR_W,
        //  // Reserved
        //  0xca => Opcode::BREAKPOINT,
        //  0xfe => Opcode::IMPDEP1,
        //  0xff => Opcode::IMPDEP2,
        e => return Err(anyhow!("unknown opcode {:#01x}", e)),
    })
}
