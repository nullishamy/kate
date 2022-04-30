use crate::{Args, CallSite, VM};
use std::collections::HashMap;

pub type NativeMethod = fn(&VM, &mut Args, &mut CallSite) -> anyhow::Result<()>;

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
