

use std::rc::Rc;

use super::{Instruction, Progression};
use crate::{
    object::numeric::{FloatingType, IntegralType},
    Context, VM, native::NativeFunction
};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use parse::{pool::ConstantEntry, classfile::Resolvable, flags::MethodAccessFlag, attributes::CodeAttribute};
use support::descriptor::MethodType;

#[derive(Debug)]
pub struct InvokeStatic {
    pub(crate) index: u16,
}

impl Instruction for InvokeStatic {
    fn handle(&self, vm: &mut VM, ctx: &mut Context) -> Result<Progression> {
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
                        "attempted to InvokeStatic an instance native method"
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

        Ok(Progression::Next)
    }
}