use crate::runtime::heap::object::JVMObject;
use crate::structs::JVMPointer;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Weak};
use tracing::{debug, trace};

pub mod object;

const MAX_HEAP: usize = 3096;

pub struct Heap {
    refs: Vec<Weak<JVMObject>>,
}

impl Heap {
    pub fn new() -> Self {
        Self { refs: vec![] }
    }

    pub fn push(&mut self, obj: JVMObject) -> Result<Arc<JVMObject>> {
        trace!("pushing `{}` onto heap", obj.class.this_class.name.str);
        if self.refs.len() >= MAX_HEAP - 1 {
            return Err(anyhow!(
                "out of memory {} objects allocated",
                self.refs.len()
            ));
        }

        let rc = Arc::new(obj);

        let weak = Arc::downgrade(&rc);
        self.refs.push(weak);

        Ok(rc)
    }

    fn prune(&mut self) -> usize {
        let mut counter = 0;

        // Iterate through the indices of weak references without any corresponding strong references
        let dead_refs: Vec<usize> = self
            .refs
            .iter()
            .enumerate()
            .filter(|(i, e)| e.strong_count() == 0)
            .map(|(i, _)| i)
            .rev()
            .collect();

        for dead in dead_refs {
            self.refs.swap_remove(dead);
            counter += 1;
        }

        // TODO: determine when we should de-allocate, this shouldnt be done often as
        // its expensive to then reallocate memory
        // self.refs.shrink_to_fit();

        debug!("pruned {counter} elements from heap");
        counter
    }

    pub fn get(&self, ptr: &JVMPointer) -> Result<Arc<JVMObject>> {
        let obj = self.refs.get(*ptr as usize);

        if obj.is_none() {
            return Err(anyhow!("could not locate object for ptr {}", ptr));
        }

        let obj = obj.unwrap().upgrade();

        if obj.is_none() {
            return Err(anyhow!("could not upgrade object for ptr {}", ptr));
        }

        Ok(obj.unwrap())
    }
}
