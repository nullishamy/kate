use std::sync::Arc;

use crate::LoadedClassFile;

pub struct JVMObject {
    pub class: Arc<LoadedClassFile>,
}
