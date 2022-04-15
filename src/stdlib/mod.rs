mod visitors;

use crate::stdlib::visitors::visit_system;
use crate::LoadedClassFile;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

type VisitFunc = fn(Arc<LoadedClassFile>);

lazy_static::lazy_static! {
    pub static ref VISITORS: HashMap<String, VisitFunc> = {
        let mut m = HashMap::new();

        m.insert("java/lang/System".to_string(), visit_system as VisitFunc);

        m
    };
}
