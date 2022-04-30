use std::collections::HashMap;
use std::sync::Arc;

use crate::stdlib::visitors::{visit_shutdown, visit_system};
use crate::LoadedClassFile;

mod visitors;

type VisitFunc = fn(Arc<LoadedClassFile>);

lazy_static::lazy_static! {
    pub static ref VISITORS: HashMap<String, VisitFunc> = {
        let mut m = HashMap::new();

        m.insert("java/lang/System".to_string(), visit_system as VisitFunc);
        m.insert("java/lang/Shutdown".to_string(), visit_shutdown as VisitFunc);

        m
    };
}
