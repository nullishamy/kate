use std::rc::Rc;

use anyhow::Result;

use parking_lot::Mutex;

use crate::runtime::object::ClassObject;

pub trait ClassLoader {
    fn load_class(&self, name: String) -> Result<Rc<Mutex<ClassObject>>>;
}
