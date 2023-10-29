#![allow(clippy::redundant_closure_call)]

use super::{Instruction, Progression};
use crate::{
    arg,
    object::{
        numeric::{Floating, FloatingType, Integral, IntegralType},
        RuntimeValue,
    },
    pop, Context, VM,
};
use anyhow::{anyhow, Context as AnyhowContext, Result};

macro_rules! binop {
    // Generic value transformation
    ($ins: ident, $lhs: ident, $rhs: ident, $res: ty, $res_trans: expr => $op: expr) => {
        #[derive(Debug)]
        pub struct $ins;

        impl Instruction for $ins {
            fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
                let rhs = arg!(ctx, "rhs" => $rhs);
                let lhs = arg!(ctx, "lhs" => $lhs);

                let result: $res = $op(lhs, rhs);
                ctx.operands.push($res_trans(result));

                Ok(Progression::Next)
            }
        }
    };
    // Generic conditional transformation
    ($ins: ident, $res_ty: ident => $op: expr) => {
        #[derive(Debug)]
        pub struct $ins {
            pub(crate) jump_to: i16
        }

        impl Instruction for $ins {
            fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
                let rhs = arg!(ctx, "rhs" => $res_ty);
                let lhs = arg!(ctx, "lhs" => $res_ty);

                let result: bool = $op(lhs, rhs);
                if (result) {
                    Ok(Progression::JumpRel(self.jump_to as i32))
                } else {
                    Ok(Progression::Next)
                }
            }
        }
    };
    ($ins: ident (int) => $op: expr) => {
      binop!($ins, i32, i32, i32, |result: i32| RuntimeValue::Integral(result.into()) => $op);
    };
    ($ins: ident (long) => $op: expr) => {
      binop!($ins, i64, i64, i64, |result: i64| RuntimeValue::Integral(result.into()) => $op);
    };
    ($ins: ident (long bitwise) => $op: expr) => {
      binop!($ins, i64, i32, i64, |result: i64| RuntimeValue::Integral(result.into()) => $op);
    };
    ($ins: ident (float) => $op: expr) => {
      binop!($ins, f32, f32, f32, |result: f32| RuntimeValue::Floating(result.into()) => $op);
    };
    ($ins: ident (double) => $op: expr) => {
      binop!($ins, f64, f64, f64, |result: f64| RuntimeValue::Floating(result.into()) => $op);
    };
    ($ins: ident (int cond) => $op: expr) => {
      binop!($ins, i32 => $op);
    };
}

// Binary (int)
binop!(Isub (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32).wrapping_sub(rhs.value as i32)
});

binop!(Iadd (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32).wrapping_add(rhs.value as i32)
});

binop!(Imul (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32).wrapping_mul(rhs.value as i32)
});

binop!(Idiv (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32).wrapping_div(rhs.value as i32)
});

binop!(Irem (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32) % (rhs.value as i32)
});

// Binary (long)
binop!(Lsub (long) => |lhs: Integral, rhs: Integral| {
    lhs.value.wrapping_sub(rhs.value)
});

binop!(Ladd (long) => |lhs: Integral, rhs: Integral| {
    lhs.value.wrapping_add(rhs.value)
});

binop!(Lmul (long) => |lhs: Integral, rhs: Integral| {
    lhs.value.wrapping_mul(rhs.value)
});

binop!(Ldiv (long) => |lhs: Integral, rhs: Integral| {
    lhs.value.wrapping_div(rhs.value)
});

binop!(Lrem (long) => |lhs: Integral, rhs: Integral| {
    lhs.value % rhs.value
});

// Binary (float)
binop!(Fadd (float) => |lhs: Floating, rhs: Floating| {
    (lhs.value as f32) + (rhs.value as f32)
});

binop!(Fsub (float) => |lhs: Floating, rhs: Floating| {
    (lhs.value as f32) - (rhs.value as f32)
});

binop!(Fmul (float) => |lhs: Floating, rhs: Floating| {
    (lhs.value as f32) * (rhs.value as f32)
});

binop!(Fdiv (float) => |lhs: Floating, rhs: Floating| {
    (lhs.value as f32) / (rhs.value as f32)
});

binop!(Frem (float) => |lhs: Floating, rhs: Floating| {
    (lhs.value as f32) % (rhs.value as f32)
});

// Binary (double)
binop!(Dadd (double) => |lhs: Floating, rhs: Floating| {
    lhs.value + rhs.value
});

binop!(Dsub (double) => |lhs: Floating, rhs: Floating| {
    lhs.value - rhs.value
});

binop!(Dmul (double) => |lhs: Floating, rhs: Floating| {
    lhs.value * rhs.value
});

binop!(Ddiv (double) => |lhs: Floating, rhs: Floating| {
    lhs.value / rhs.value
});

binop!(Drem (double) => |lhs: Floating, rhs: Floating| {
    lhs.value % rhs.value
});

// Conditional (int)
binop!(Ieq (int cond) => |lhs: Integral, rhs: Integral| {
    lhs.value == rhs.value
});

binop!(Ine (int cond) => |lhs: Integral, rhs: Integral| {
    lhs.value != rhs.value
});

binop!(Ile (int cond) => |lhs: Integral, rhs: Integral| {
    lhs.value <= rhs.value
});

binop!(Ige (int cond) => |lhs: Integral, rhs: Integral| {
    lhs.value >= rhs.value
});

binop!(Igt (int cond) => |lhs: Integral, rhs: Integral| {
    lhs.value > rhs.value
});

binop!(Ilt (int cond) => |lhs: Integral, rhs: Integral| {
    lhs.value < rhs.value
});

// Bitwise
binop!(Lshl (long bitwise) => |lhs: Integral, rhs: Integral| {
    lhs.value.wrapping_shl(rhs.value as u32)
});

binop!(Lshr (long bitwise) => |lhs: Integral, rhs: Integral| {
    lhs.value.wrapping_shr(rhs.value as u32)
});

binop!(Land (long) => |lhs: Integral, rhs: Integral| {
    lhs.value & rhs.value
});

binop!(Lushr (long bitwise) => |lhs: Integral, rhs: Integral| {
    ((lhs.value as u64) >> (rhs.value as u64)) as i64
});

binop!(Ishl (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32).wrapping_shl(rhs.value as u32)
});

binop!(Ishr (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32).wrapping_shr(rhs.value as u32)
});

binop!(Iushr (int) => |lhs: Integral, rhs: Integral| {
    ((lhs.value as u32) >> (rhs.value as u32)) as i32
});

binop!(Iand (int) => |lhs: Integral, rhs: Integral| {
    (lhs.value as i32) & (rhs.value as i32)
});
