use std::fmt;

use crate::{object::RuntimeValue, Context, VM};
use anyhow::{anyhow, Result, Error};
use bytes::BytesMut;
use support::bytes_ext::SafeBuf;

mod binary;
mod invoke;
mod load_store;
mod ops;
mod unary;

pub enum Progression {
    JumpAbs(i32),
    JumpRel(i32),
    Next,
    Return(Option<RuntimeValue>),
    Throw(Error)
}

pub trait Instruction: fmt::Debug {
    fn handle(&self, _vm: &mut VM, _ctx: &mut Context) -> Result<Progression> {
        Ok(Progression::Next)
    }
}

/// Utility to box a value. Used below to box each instruction that we decode
fn b<T>(v: T) -> Box<T> {
    Box::new(v)
}

pub fn decode_instruction(_vm: &VM, bytes: &mut BytesMut) -> Result<Box<dyn Instruction>> {
    let instruction = bytes.try_get_u8()?;
    Ok(match instruction {
        0x00 => b(ops::Nop),

        // Constants / Loads / Stores
        0x01 => b(ops::PushConst {
            value: RuntimeValue::Null,
        }),
        0x02 => b(ops::PushConst {
            value: RuntimeValue::Integral((-1_i32).into()),
        }),
        0x03 => b(ops::PushConst {
            value: RuntimeValue::Integral((0_i32).into()),
        }),
        0x04 => b(ops::PushConst {
            value: RuntimeValue::Integral((1_i32).into()),
        }),
        0x05 => b(ops::PushConst {
            value: RuntimeValue::Integral((2_i32).into()),
        }),
        0x06 => b(ops::PushConst {
            value: RuntimeValue::Integral((3_i32).into()),
        }),
        0x07 => b(ops::PushConst {
            value: RuntimeValue::Integral((4_i32).into()),
        }),
        0x08 => b(ops::PushConst {
            value: RuntimeValue::Integral((5_i32).into()),
        }),

        0x09 => b(ops::PushConst {
            value: RuntimeValue::Integral((0_i64).into()),
        }),
        0x0a => b(ops::PushConst {
            value: RuntimeValue::Integral((1_i64).into()),
        }),
        0x0b => b(ops::PushConst {
            value: RuntimeValue::Floating((0.0_f32).into()),
        }),
        0x0c => b(ops::PushConst {
            value: RuntimeValue::Floating((1.0_f32).into()),
        }),
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
            value: RuntimeValue::Integral((bytes.try_get_i8()? as i32).into()),
        }),
        0x11 => b(ops::PushConst {
            // The intermediate value is then sign-extended to an int value.
            value: RuntimeValue::Integral((bytes.try_get_i16()? as i32).into()),
        }),
        0x12 => b(ops::Ldc {
            index: bytes.try_get_u8()? as u16,
        }),
        0x13 => b(ops::Ldc {
            index: bytes.try_get_u8()? as u16,
        }),
        0x14 => b(ops::Ldc2W {
            index: bytes.try_get_u16()?,
        }),
        0x15 => b(ops::LoadLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        0x16 => b(ops::LoadLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        0x17 => b(ops::LoadLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        0x18 => b(ops::LoadLocal {
            index: bytes.try_get_u8()? as usize,
        }),
        0x19 => b(ops::LoadLocal {
            index: bytes.try_get_u8()? as usize,
        }),

        0x1a => b(ops::LoadLocal { index: 0 }),
        0x1b => b(ops::LoadLocal { index: 1 }),
        0x1c => b(ops::LoadLocal { index: 2 }),
        0x1d => b(ops::LoadLocal { index: 3 }),

        0x1e => b(ops::LoadLocal { index: 0 }),
        0x1f => b(ops::LoadLocal { index: 1 }),
        0x20 => b(ops::LoadLocal { index: 2 }),
        0x21 => b(ops::LoadLocal { index: 3 }),

        0x22 => b(ops::LoadLocal { index: 0 }),
        0x23 => b(ops::LoadLocal { index: 1 }),
        0x24 => b(ops::LoadLocal { index: 2 }),
        0x25 => b(ops::LoadLocal { index: 3 }),

        0x26 => b(ops::LoadLocal { index: 0 }),
        0x27 => b(ops::LoadLocal { index: 1 }),
        0x28 => b(ops::LoadLocal { index: 2 }),
        0x29 => b(ops::LoadLocal { index: 3 }),

        0x2a => b(ops::LoadLocal { index: 0 }),
        0x2b => b(ops::LoadLocal { index: 1 }),
        0x2c => b(ops::LoadLocal { index: 2 }),
        0x2d => b(ops::LoadLocal { index: 3 }),

        0x2e => b(ops::ArrayLoad),
        0x2f => b(ops::ArrayLoad),
        0x30 => b(ops::ArrayLoad),
        0x31 => b(ops::ArrayLoad),
        0x32 => b(ops::ArrayLoad),
        0x33 => b(ops::ArrayLoad),
        0x34 => b(ops::ArrayLoad),
        0x35 => b(ops::ArrayLoad),

        0x36 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
            store_next: false,
        }),
        0x37 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
            store_next: true,
        }),
        0x38 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
            store_next: false,
        }),
        0x39 => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
            store_next: true,
        }),
        0x3a => b(ops::StoreLocal {
            index: bytes.try_get_u8()? as usize,
            store_next: false,
        }),
        0x3b => b(ops::StoreLocal {
            index: 0,
            store_next: false,
        }),
        0x3c => b(ops::StoreLocal {
            index: 1,
            store_next: false,
        }),
        0x3d => b(ops::StoreLocal {
            index: 2,
            store_next: false,
        }),
        0x3e => b(ops::StoreLocal {
            index: 3,
            store_next: false,
        }),

        0x3f => b(ops::StoreLocal {
            index: 0,
            store_next: true,
        }),
        0x40 => b(ops::StoreLocal {
            index: 1,
            store_next: true,
        }),
        0x41 => b(ops::StoreLocal {
            index: 2,
            store_next: true,
        }),
        0x42 => b(ops::StoreLocal {
            index: 3,
            store_next: true,
        }),

        0x43 => b(ops::StoreLocal {
            index: 0,
            store_next: false,
        }),
        0x44 => b(ops::StoreLocal {
            index: 1,
            store_next: false,
        }),
        0x45 => b(ops::StoreLocal {
            index: 2,
            store_next: false,
        }),
        0x46 => b(ops::StoreLocal {
            index: 3,
            store_next: false,
        }),

        0x47 => b(ops::StoreLocal {
            index: 0,
            store_next: false,
        }),
        0x48 => b(ops::StoreLocal {
            index: 1,
            store_next: false,
        }),
        0x49 => b(ops::StoreLocal {
            index: 2,
            store_next: false,
        }),
        0x4a => b(ops::StoreLocal {
            index: 3,
            store_next: true,
        }),

        0x4b => b(ops::StoreLocal {
            index: 0,
            store_next: false,
        }),
        0x4c => b(ops::StoreLocal {
            index: 1,
            store_next: false,
        }),
        0x4d => b(ops::StoreLocal {
            index: 2,
            store_next: false,
        }),
        0x4e => b(ops::StoreLocal {
            index: 3,
            store_next: false,
        }),

        0x4f => b(ops::ArrayStore),
        0x50 => b(ops::ArrayStore),
        0x51 => b(ops::ArrayStore),
        0x52 => b(ops::ArrayStore),
        0x53 => b(ops::ArrayStore),
        0x54 => b(ops::ArrayStore),
        0x55 => b(ops::ArrayStore),
        0x56 => b(ops::ArrayStore),

        // Stack Math Conversions
        0x57 => b(ops::Pop { amount: 1 }),
        0x58 => b(ops::Pop { amount: 2 }),
        0x59 => b(ops::Dup),
        //  0x5a => Opcode::DUP_X1,
        //  0x5b => Opcode::DUP_X2,
        //  0x5c => Opcode::DUP2,
        //  0x5d => Opcode::DUP2_X1,
        //  0x5e => Opcode::DUP2_X2,
        //  0x5f => Opcode::SWAP,
        0x60 => b(ops::Iadd),
        0x61 => b(ops::Ladd),
        0x62 => b(ops::Fadd),
        0x63 => b(ops::Dadd),
        0x64 => b(ops::Isub),
        0x65 => b(ops::Lsub),
        0x66 => b(ops::Fsub),
        0x67 => b(ops::Dsub),
        0x68 => b(ops::Imul),
        0x69 => b(ops::Lmul),
        0x6a => b(ops::Fmul),
        0x6b => b(ops::Dmul),
        0x6c => b(ops::Idiv),
        0x6d => b(ops::Ldiv),
        0x6e => b(ops::Fdiv),
        0x6f => b(ops::Ddiv),
        0x70 => b(ops::Irem),
        0x71 => b(ops::Lrem),
        0x72 => b(ops::Frem),
        0x73 => b(ops::Drem),
        0x74 => b(ops::Ineg),
        0x75 => b(ops::Lneg),
        0x76 => b(ops::Fneg),
        0x77 => b(ops::Dneg),
        0x78 => b(ops::Ishl),
        0x79 => b(ops::Lshl),
        0x7a => b(ops::Ishr),
        0x7b => b(ops::Lshr),
        0x7c => b(ops::Iushr),
        0x7d => b(ops::Lushr),
        //  0x7e => Opcode::IAND,
        //  0x7f => Opcode::LAND,
        //  0x80 => Opcode::IOR,
        //  0x81 => Opcode::LOR,
        //  0x82 => Opcode::IXOR,
        //  0x83 => Opcode::LXOR,
        //  0x84 => Opcode::IINC(bytes.try_get_u8()?, bytes.try_get_i8()?),
        0x85 => b(ops::I2l),
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

        // Comparisons
        //  0x94 => Opcode::LCMP,
        //  0x95 => Opcode::FCMPL,
        //  0x96 => Opcode::FCMPG,
        //  0x97 => Opcode::DCMPL,
        //  0x98 => Opcode::DCMPG,
        0x99 => b(ops::IfEq {
            jump_to: bytes.try_get_i16()?,
        }),
        0x9a => b(ops::IfNe {
            jump_to: bytes.try_get_i16()?,
        }),
        0x9b => b(ops::IfLt {
            jump_to: bytes.try_get_i16()?,
        }),
        0x9c => b(ops::IfGe {
            jump_to: bytes.try_get_i16()?,
        }),
        0x9d => b(ops::IfGt {
            jump_to: bytes.try_get_i16()?,
        }),
        0x9e => b(ops::IfLe {
            jump_to: bytes.try_get_i16()?,
        }),
        0x9f => b(ops::Ieq {
            jump_to: bytes.try_get_i16()?,
        }),
        0xa0 => b(ops::Ine {
            jump_to: bytes.try_get_i16()?,
        }),
        0xa1 => b(ops::Ilt {
            jump_to: bytes.try_get_i16()?,
        }),
        0xa2 => b(ops::Ige {
            jump_to: bytes.try_get_i16()?,
        }),
        0xa3 => b(ops::Igt {
            jump_to: bytes.try_get_i16()?,
        }),
        0xa4 => b(ops::Ile {
            jump_to: bytes.try_get_i16()?,
        }),
        //  0xa5 => Opcode::IF_ACMPEQ(bytes.try_get_i16()?),
        //  0xa6 => Opcode::IF_ACMPNE(bytes.try_get_i16()?),

        // Control
        0xa7 => b(ops::Goto {
            jump_to: bytes.try_get_i16()?,
        }),
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
        0xac => b(ops::ValueReturn),
        0xad => b(ops::ValueReturn),
        0xae => b(ops::ValueReturn),
        0xaf => b(ops::ValueReturn),
        0xb0 => b(ops::ValueReturn),
        0xb1 => b(ops::VoidReturn),

        // References
        0xb2 => b(ops::GetStatic {
            index: bytes.try_get_u16()?,
        }),
        0xb3 => b(ops::PutStatic {
            index: bytes.try_get_u16()?,
        }),
        0xb4 => b(ops::GetField {
            index: bytes.try_get_u16()?,
        }),
        0xb5 => b(ops::PutField {
            index: bytes.try_get_u16()?,
        }),
        0xb6 => b(ops::InvokeVirtual {
            index: bytes.try_get_u16()?,
        }),
        0xb7 => b(ops::InvokeSpecial {
            index: bytes.try_get_u16()?,
        }),
        0xb8 => b(ops::InvokeStatic {
            index: bytes.try_get_u16()?,
        }),
        //  0xb9 => Opcode::INVOKEINTERFACE(
        //      bytes.try_get_u16()?,
        //      bytes.try_get_u8()?,
        //      bytes.try_get_u8()?,
        //  ),
        //  0xba => Opcode::INVOKEDYNAMIC,
        0xbb => b(ops::New {
            index: bytes.try_get_u16()?,
        }),
        0xbc => b(ops::NewArray {
            type_tag: bytes.try_get_u8()?,
        }),
        0xbd => b(ops::ANewArray {
            type_index: bytes.try_get_u16()?,
        }),
        0xbe => b(ops::ArrayLength),
        0xbf => b(ops::Athrow),
        //  0xc0 => Opcode::CHECKCAST(bytes.try_get_u16()?),
        //  0xc1 => Opcode::INSTANCEOF(bytes.try_get_u16()?),
        //  0xc2 => Opcode::MONITORENTER,
        //  0xc3 => Opcode::MONITOREXIT,

        // Extended
        //  0xc4 => Opcode::WIDE,
        //  0xc5 => Opcode::MULTIANEWARRAY,
        0xc6 => b(ops::IfNull { jump_to: bytes.try_get_i16()? }),
        0xc7 => b(ops::IfNotNull { jump_to: bytes.try_get_i16()? }),
        //  0xc8 => Opcode::GOTO_W,
        //  0xc9 => Opcode::JSR_W,

        // Reserved
        0xca => b(ops::Nop),
        0xfe => b(ops::Nop),
        0xff => b(ops::Nop),
        e => return Err(anyhow!("unknown opcode {:#01x}", e)),
    })
}
