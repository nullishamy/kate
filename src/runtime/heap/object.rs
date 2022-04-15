


use crate::LoadedClassFile;


use std::sync::Arc;

pub struct JVMObject {
    pub class: Arc<LoadedClassFile>,
}
