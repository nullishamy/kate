use std::sync::Arc;

use crate::LoadedClassFile;

#[derive(Clone, Debug)]
pub struct JVMObject {
    pub class: Arc<LoadedClassFile>,
}
