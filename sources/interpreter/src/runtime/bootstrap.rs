use std::{collections::HashMap, fs, path::PathBuf, rc::Rc, sync::RwLock};

use crate::{
    runtime::{classloader::ClassLoader, object::ClassObject},
};
use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use parse::parser::Parser;



pub struct BootstrapClassLoader {
    class_paths: Vec<PathBuf>,
    loaded_classes: RwLock<HashMap<String, Rc<Mutex<ClassObject>>>>,
}

impl ClassLoader for BootstrapClassLoader {
    fn load_class(&self, name: String) -> Result<Rc<Mutex<ClassObject>>> {
        let name = format!("{}.class", name);

        if let Some(object) = self.loaded_classes.read().unwrap().get(&name) {
            return Ok(Rc::clone(object));
        }

        let mut found_path: Option<PathBuf> = None;

        for root in self.class_paths.iter() {
            let path = root.join::<PathBuf>(name.clone().into());
            if path.exists() {
                found_path = Some(path);
                break;
            }
        }

        if let Some(path) = found_path {
            let bytes = fs::read(path)?;
            let mut parser = Parser::new(&bytes);

            let object = Rc::new(Mutex::new(ClassObject::for_runtime_object(parser.parse()?)));
            self.loaded_classes
                .write()
                .unwrap()
                .insert(name.clone(), Rc::clone(&object));

            return Ok(object);
        }

        Err(anyhow!("Could not locate classfile {}", name))
    }
}

impl BootstrapClassLoader {
    pub fn new(class_paths: Vec<PathBuf>) -> Self {
        Self {
            class_paths,
            loaded_classes: RwLock::new(HashMap::new()),
        }
    }
}
