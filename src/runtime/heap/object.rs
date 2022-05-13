use std::sync::Arc;

use crate::LoadedClassFile;

#[derive(Clone, Debug)]
pub struct JvmObject {
    pub class: Arc<LoadedClassFile>,
}
