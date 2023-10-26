use std::rc::Rc;

use parking_lot::RwLock;
use anyhow::Result;

use super::{WrappedClassObject, StringObject};

// TODO: Actually intern
pub struct Interner {
  string_class: WrappedClassObject
}

impl Interner {
  pub fn new (string_class: WrappedClassObject) -> Self {
    Self {
      string_class
    }
  }

  pub fn intern(&mut self, s: String) -> Result<Rc<RwLock<StringObject>>> {
    let obj = StringObject::new(Rc::clone(&self.string_class), s)?;
    Ok(Rc::new(RwLock::new(obj)))
  }
}