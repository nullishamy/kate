use crate::{Args, CallSite, Vm};
use std::collections::HashMap;

pub type NativeMethod = fn(&Vm, &mut Args, &mut CallSite) -> anyhow::Result<()>;

pub struct NativeMethodController {
    pub entries: HashMap<String, NativeMethod>,
}

impl NativeMethodController {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}
