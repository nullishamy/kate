use crate::structs::loaded::constant_pool::Utf8Data;
use crate::structs::loaded::field::Fields;
use crate::structs::loaded::method::Methods;
use crate::LoadedClassFile;
use parking_lot::RwLock;
use std::rc::Rc;
use std::sync::Arc;

pub struct JVMObject {
    pub class: Arc<LoadedClassFile>,
}
