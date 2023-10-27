
use super::{Instruction, Progression};
use crate::{
    object::{
        numeric::{FloatingType, IntegralType, Integral, Floating},
        RuntimeValue,
    },
    Context, VM, arg, pop
};
use anyhow::{anyhow, Context as AnyhowContext, Result};

macro_rules! unop {
    // Generic value transformation 
    ($ins: ident, $res_ty: ident, $res_trans: expr => $op: expr) => {
        #[derive(Debug)]
        pub struct $ins;

        impl Instruction for $ins {
            fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
                let val = arg!(ctx, "unary value" => $res_ty);

                let result = $op(val);
                ctx.operands.push($res_trans(result));

                Ok(Progression::Next)
            }
        }
    };
    // Generic duplicated value transformation 
    (x2 $ins: ident, $res_ty: ident, $res_trans: expr => $op: expr) => {
        #[derive(Debug)]
        pub struct $ins;

        impl Instruction for $ins {
            fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
                let val = arg!(ctx, "unary value" => $res_ty);

                let result: $res_ty = $op(val);
                ctx.operands.push($res_trans(result));
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
      unop!($ins, i32, |result: i32| RuntimeValue::Integral(result.into()) => $op);
    };
    ($ins: ident (int => long) => $op: expr) => {
      unop!($ins, i32, |result: i64| RuntimeValue::Integral(result.into()) => $op);
    };
    ($ins: ident (long) => $op: expr) => {
      unop!(x2 $ins, i64, |result: i64| RuntimeValue::Integral(result.into()) => $op);
    };
    ($ins: ident (float) => $op: expr) => {
      unop!($ins, f32, |result: f32| RuntimeValue::Floating(result.into()) => $op);
    };
    ($ins: ident (double) => $op: expr) => {
      unop!($ins, f64, |result: f64| RuntimeValue::Floating(result.into()) => $op);
    };
}

// Negations
unop!(Ineg (int) => |val: Integral| {
    (val.value as i32).wrapping_neg()
});

unop!(Lneg (long) => |val: Integral| {
    val.value.wrapping_neg()
});

unop!(Fneg (float) => |val: Floating| {
    -(val.value as f32)
});

unop!(Dneg (double) => |val: Floating| {
    -val.value
});

// Conversions
unop!(I2l (int => long) => |val: Integral| {
    val.value as i64
});
