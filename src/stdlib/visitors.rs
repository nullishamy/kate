use crate::runtime::heap::object::JVMObject;
use crate::structs::loaded::field::Fields;
use crate::structs::loaded::method::Methods;
use crate::LoadedClassFile;
use parking_lot::lock_api::RwLock;
use std::rc::Rc;
use std::sync::Arc;

pub fn visit_system(class: Arc<LoadedClassFile>) {
    //FIXME: add the system values here

    // class
    //     .fields
    //     .write()
    //     .statics
    //     .insert("out".to_string(), Rc::new_zeroed());
}
