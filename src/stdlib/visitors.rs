use std::sync::Arc;

use crate::LoadedClassFile;

pub fn visit_system(_class: Arc<LoadedClassFile>) {
    //FIXME: add the system values here

    // class
    //     .fields
    //     .write()
    //     .statics
    //     .insert("out".to_string(), Rc::new_zeroed());
}
