use std::rc::Rc;

use super::Instruction;
use crate::{
    native::NativeFunction,
    object::{
        numeric::{FloatingType, IntegralType},
        RuntimeValue,
    },
    Context, VM,
};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use parse::{
    attributes::CodeAttribute, classfile::Resolvable, flags::MethodAccessFlag, pool::ConstantEntry,
};
use support::descriptor::MethodType;

macro_rules! nop {
  ($( $x:ident ),* ) => {
    $(
      #[derive(Debug)]
      pub struct $x {}
      impl Instruction for $x { }
    )*
  };
}

macro_rules! pop {
    ($ctx: expr) => {
        $ctx
            .operands
            .pop()
            .context("no value to pop from the operand stack")?
    };
}

macro_rules! arg {
    ($ctx: expr, $side: expr => int) => {{
        let val = pop!($ctx);

        let val = val.as_integral().context(format!("{} was not an integral", $side))?;
        if val.ty != IntegralType::Int {
            return Err(anyhow!(format!("{} was not an int", $side)));
        }

        val.clone()
    }}
}

nop!(Nop, Return);

#[derive(Debug)]
pub struct PushConst {
    pub(crate) value: RuntimeValue,
}

impl Instruction for PushConst {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        ctx.operands.push(self.value.clone());
        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct Ldc2W {
    pub(crate) index: u16,
}

impl Instruction for Ldc2W {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let value = ctx
            .class
            .read()
            .constant_pool()
            .address(self.index)
            .try_resolve()
            .context(format!("no value @ index {}", self.index))?;

        match value {
            ConstantEntry::Long(data) => {
                ctx.operands
                    .push(RuntimeValue::Integral((data.bytes as i64).into()));
            }
            ConstantEntry::Double(data) => {
                ctx.operands.push(RuntimeValue::Floating(data.bytes.into()));
            }
            v => return Err(anyhow!("cannot load {:#?} with ldc2w", v)),
        };

        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct Ldc {
    pub(crate) index: u8,
}

impl Instruction for Ldc {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let value = ctx
            .class
            .read()
            .constant_pool()
            .address(self.index as u16)
            .try_resolve()
            .context(format!("no value @ index {}", self.index))?;

        match value {
            ConstantEntry::Integer(data) => {
                ctx.operands
                    .push(RuntimeValue::Integral((data.bytes as i32).into()));
            }
            ConstantEntry::Float(data) => {
                ctx.operands.push(RuntimeValue::Floating(data.bytes.into()));
            }
            v => return Err(anyhow!("cannot load {:#?} with ldc", v)),
        };

        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct Isub;

impl Instruction for Isub {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let rhs = arg!(ctx, "rhs" => int);
        let lhs = arg!(ctx, "lhs" => int);

        let result: i32 = (lhs.value as i32).wrapping_sub(rhs.value as i32);
        ctx.operands.push(RuntimeValue::Integral(result.into()));

        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct Irem;

impl Instruction for Irem {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let rhs = arg!(ctx, "rhs" => int);
        let lhs = arg!(ctx, "lhs" => int);

        dbg!(&lhs);
        dbg!(&rhs);

        let result: i32 = (lhs.value as i32) % (rhs.value as i32);
        ctx.operands.push(RuntimeValue::Integral(result.into()));

        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct LoadLocal {
    pub(crate) index: usize,
}

impl Instruction for LoadLocal {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let local = ctx
            .locals
            .get(self.index)
            .context(format!("no local @ {}", self.index))?;
        ctx.operands.push(local.clone());
        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct StoreLocal {
    pub(crate) index: usize,
}

impl Instruction for StoreLocal {
    fn handle(&self, _vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let locals = &mut ctx.locals;
        let index = self.index;
        let value = ctx.operands.pop().context("no operand to pop")?.clone();

        // Fill enough slots to be able to store at an arbitrary index
        // FIXME: We should probably keep a track of which locals are filled with "real"
        // values and which are just sentinels so we can provide more accurate diagnostics
        // for invalid store / get ops
        while locals.len() <= index {
            locals.push(RuntimeValue::Null);
        }

        locals[index] = value;
        Ok(ctx.pc)
    }
}

#[derive(Debug)]
pub struct InvokeStatic {
    pub(crate) index: u16,
}

impl Instruction for InvokeStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<i32> {
        let cls = ctx.class.read();
        let pool = cls.constant_pool();

        // The unsigned indexbyte1 and indexbyte2 are used to construct an
        // index into the run-time constant pool of the current class (§2.6),
        let pool_entry = pool
            .address::<ConstantEntry>(self.index)
            .try_resolve()
            .context(format!("no method at index {}", self.index))?;

        drop(cls);

        // The run-time constant pool entry at the index must be a symbolic
        // reference to a method or an interface method (§5.1), which gives
        // the name and descriptor (§4.3.3) of the method or interface method
        // as well as a symbolic reference to the class or interface in which
        // the method or interface method is to be found.
        let (method_name, method_descriptor, class_name) = match pool_entry {
            ConstantEntry::Method(data) => {
                let name_and_type = data.name_and_type.resolve();
                let method_name = name_and_type.name.resolve().try_string()?;

                let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
                let method_descriptor = MethodType::parse(method_descriptor)?;

                let class = data.class.resolve();
                let class = class.name.resolve().try_string()?;

                (method_name, method_descriptor, class)
            }
            ConstantEntry::InterfaceMethod(data) => {
                let name_and_type = data.name_and_type.resolve();
                let method_name = name_and_type.name.resolve().try_string()?;

                let method_descriptor = name_and_type.descriptor.resolve().try_string()?;
                let method_descriptor = MethodType::parse(method_descriptor)?;

                let class = data.class.resolve();
                let class = class.name.resolve().try_string()?;

                (method_name, method_descriptor, class)
            }
            e => return Err(anyhow!("expected interface method / method, got {:#?}", e)),
        };

        // The named method is resolved (§5.4.3.3, §5.4.3.4).
        let loaded_class = vm.class_loader.load_class(class_name.clone())?;
        let loaded_method = loaded_class
            .read()
            .get_method((method_name.clone(), method_descriptor.to_string()))
            .context(format!("no method {} in {}", method_name, class_name))?;

        // The resolved method must not be an instance initialization method,
        // or the class or interface initialization method (§2.9.1, §2.9.2).
        if method_name == "<clinit>" || method_name == "<init>" {
            return Err(anyhow!(
                "cannot call static method {}, it is a class initialisation method",
                method_name
            ));
        }

        // On successful resolution of the method, the class or interface that
        // declared the resolved method is initialized if that class or interface
        // has not already been initialized (§5.5).
        vm.initialise_class(Rc::clone(&loaded_class))?;

        // If the method is not native, the nargs argument values are popped
        // from the operand stack. A new frame is created on the Java Virtual
        // Machine stack for the method being invoked. The nargs argument
        // values are consecutively made the values of local variables of the
        // new frame, with arg1 in local variable 0 (or, if arg1 is of type
        // long or double, in local variables 0 and 1) and so on.
        let mut reversed_descriptor = method_descriptor.clone();
        reversed_descriptor.parameters.reverse();
        let mut args_for_call = Vec::new();
        for _arg in reversed_descriptor.parameters.iter() {
            // TODO: Validate against FieldType in descriptor
            let arg = ctx.operands.pop().ok_or(anyhow!("not enough args"))?;
            if let Some(int) = arg.as_integral() {
                if int.ty == IntegralType::Long {
                    args_for_call.push(arg.clone());
                }
            }

            if let Some(float) = arg.as_floating() {
                if float.ty == FloatingType::Double {
                    args_for_call.push(arg.clone());
                }
            }
            args_for_call.push(arg.clone());
        }

        args_for_call.reverse();

        let exec_result = if !loaded_method.flags.has(MethodAccessFlag::NATIVE) {
            // Must load the context if and only if the method is not native.
            // Native methods do not have a code attribute.
            let code = loaded_method
                .attributes
                .known_attribute::<CodeAttribute>(loaded_class.read().constant_pool())?;

            let new_context = Context {
                code,
                class: Rc::clone(&loaded_class),
                pc: 0,
                operands: vec![],
                locals: args_for_call,
            };

            // The new frame is then made current, and the Java Virtual Machine pc is set
            // to the opcode of the first instruction of the method to be invoked.
            // Execution continues with the first instruction of the method.
            vm.run(new_context)
        } else {
            let lookup = loaded_class
                .read()
                .fetch_native((method_name.clone(), method_descriptor.to_string()))
                .ok_or(anyhow!(
                    "no native method {} {:?} {} / {}",
                    class_name,
                    loaded_method.flags.flags,
                    method_name,
                    method_descriptor.to_string()
                ))?;

            match lookup {
                NativeFunction::Static(func) => func(Rc::clone(&loaded_class), args_for_call, vm),
                _ => {
                    return Err(anyhow!(
                        "attempted to INVOKESTATIC an instance native method"
                    ))
                }
            }
        };

        if let Err(e) = exec_result {
            return Err(e.context(format!(
                "when interpreting {} in {}",
                method_name, class_name
            )));
        }

        // Caller gave us a value, push it to our stack (Xreturn does this)
        if let Some(return_value) = exec_result.unwrap() {
            ctx.operands.push(return_value);
        }

        Ok(ctx.pc)
    }
}
