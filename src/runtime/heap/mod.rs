use std::sync::{Arc, Weak};

use anyhow::{anyhow, Result};
use tracing::{debug, trace};

use crate::interface::tui::UpdateHeap;
use crate::runtime::heap::object::JvmObject;
use crate::structs::JvmPointer;
use crate::{TuiCommand, TuiWriter};

pub mod object;

const MAX_HEAP: usize = 3096;

pub struct Heap {
    refs: Vec<Weak<JvmObject>>,
    used: usize,
    tui: Option<TuiWriter>,
}

impl Heap {
    pub fn new(tui: Option<TuiWriter>) -> Self {
        Self {
            // start with a quarter of the max heap, this avoids un-needed allocations
            // at program start
            refs: Vec::with_capacity(MAX_HEAP / 4),
            used: 0,
            tui,
        }
    }

    pub fn push(&mut self, obj: JvmObject) -> Result<Arc<JvmObject>> {
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
        self.used += 1;

        if let Some(tui) = &self.tui {
            tui.send(TuiCommand::Heap(UpdateHeap {
                new_size: self.used(),
                total: self.total(),
            }))
            .unwrap();
        }

        Ok(rc)
    }

    fn prune(&mut self) -> usize {
        let mut counter = 0;

        // Iterate through the indices of weak references without any corresponding strong references
        let dead_refs: Vec<usize> = self
            .refs
            .iter()
            .enumerate()
            .filter(|(_i, e)| e.strong_count() == 0)
            .map(|(i, _)| i)
            .rev()
            .collect();

        for dead in dead_refs {
            self.refs.swap_remove(dead);
            counter += 1;
        }

        // even though we still have the allocation, we dont own the value
        // and it cannot be used, so we consider it unused
        self.used -= counter;

        // TODO: determine when we should de-allocate, this shouldnt be done often as
        // its expensive to then reallocate memory
        // self.refs.shrink_to_fit();

        if let Some(tui) = &self.tui {
            tui.send(TuiCommand::Heap(UpdateHeap {
                new_size: self.used(),
                total: self.total(),
            }))
            .unwrap();
        }

        debug!("pruned {counter} elements from heap");
        counter
    }

    pub fn get(&self, ptr: &JvmPointer) -> Result<Arc<JvmObject>> {
        let obj = self.refs.get(*ptr as usize);
        let obj = obj
            .ok_or_else(|| anyhow!("could not locate object for ptr {}", ptr))?
            .upgrade();
        obj.ok_or_else(|| anyhow!("could not upgrade object for ptr {}", ptr))
    }

    pub fn total(&self) -> usize {
        // using capacity as this shows the true capacity of the vec
        // rather than how much we've used
        self.refs.capacity()
    }

    pub fn used(&self) -> usize {
        self.used
    }
}
