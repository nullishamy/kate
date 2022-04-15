use crate::structs::descriptor::MethodDescriptor;
use crate::structs::loaded::method::{MethodEntry, Methods};
use std::rc::Rc;
use std::sync::Arc;

pub struct Constructors {
    pub entries: Vec<Rc<Constructor>>,
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
            .map(Rc::new)
            .collect();

        Self { entries }
    }

    pub fn find<F>(&self, predicate: F) -> Option<Rc<Constructor>>
    where
        F: Fn(&&Rc<Constructor>) -> bool,
    {
        Some(Rc::clone(self.entries.iter().find(predicate)?))
    }
}
pub struct Constructor {
    pub descriptor: MethodDescriptor,
    pub method: Arc<MethodEntry>,
}
