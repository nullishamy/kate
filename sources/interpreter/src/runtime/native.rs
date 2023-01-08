use std::rc::Rc;

use crate::{
    interpreter::Interpreter,
    runtime::{
        object::{ClassObject, JavaObject},
        stack::RuntimeValue,
    },
};
use anyhow::Result;
use parking_lot::Mutex;

pub type NativeStaticFunction = fn(
    class: Rc<Mutex<ClassObject>>,
    args: Vec<RuntimeValue>,
    interpreter: &mut Interpreter,
) -> Result<Option<RuntimeValue>>;
pub type NativeInstanceFunction = fn(
    this: JavaObject,
    args: Vec<RuntimeValue>,
    interpreter: &mut Interpreter,
) -> Result<Option<RuntimeValue>>;

#[derive(Clone, Debug)]
pub enum NativeFunction {
    Static(NativeStaticFunction),
    Instance(NativeInstanceFunction),
}
