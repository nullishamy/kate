use std::sync::Arc;

use crate::structs::descriptor::MethodDescriptor;
use crate::structs::loaded::method::{MethodEntry, Methods};

#[derive(Debug)]
pub struct Constructors {
    pub entries: Vec<Arc<Constructor>>,
}

impl Constructors {
    pub fn from_methods(methods: &Methods) -> Self {
        let entries = methods
            .entries
            .iter()
            .filter(|x| x.name.str == "<init>")
            .map(|x| Constructor {
                // descriptors are cheap to clone, we dont need to rc/borrow these
                descriptor: x.descriptor.clone(),
                method: Arc::clone(x),
            })
            .map(Arc::new)
            .collect();

        Self { entries }
    }

    pub fn find<F>(&self, predicate: F) -> Option<Arc<Constructor>>
    where
        F: Fn(&&Arc<Constructor>) -> bool,
    {
        Some(Arc::clone(self.entries.iter().find(predicate)?))
    }
}

#[derive(Debug)]
pub struct Constructor {
    pub descriptor: MethodDescriptor,
    pub method: Arc<MethodEntry>,
}
