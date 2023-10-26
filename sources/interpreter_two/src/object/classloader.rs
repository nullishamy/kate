use std::{collections::HashMap, fs, path::PathBuf, rc::Rc};

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use parse::{parser::Parser, classfile::Resolvable};
use tracing::debug;

use super::{ClassObject, WrappedClassObject};

pub struct ClassLoader {
    classes: HashMap<String, WrappedClassObject>,
    class_path: Vec<PathBuf>,
    meta_class_object: Option<WrappedClassObject>
}

impl ClassLoader {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            class_path: vec![],
            meta_class_object: None,
        }
    }

    pub fn add_path(&mut self, path: PathBuf) -> &mut Self {
        self.class_path.push(path);
        self
    }

    pub fn load_from_bytes(&mut self, name: String, bytes: &[u8]) -> Result<WrappedClassObject> {
            let mut parser = Parser::new(&bytes);
            let classfile = parser.parse()?;

            let object = Rc::new(RwLock::new(ClassObject::new(
                self.meta_class_object.as_ref().cloned(),
                classfile.methods,
                classfile.constant_pool,
                classfile.this_class.resolve().name.resolve().try_string()?
            )));

            self.classes.insert(name.clone(), Rc::clone(&object));

            Ok(object)
    }

    pub fn load_class(&mut self, name: String) -> Result<WrappedClassObject> {
        let name = format!("{}.class", name);

        if let Some(object) = self.classes.get(&name) {
            debug!("Fast path: {}", name);
            return Ok(Rc::clone(&object));
        }

        debug!("Slow path: {}", name);

        let mut found_path: Option<PathBuf> = None;

        for root in self.class_path.iter() {
            let path = root.join::<PathBuf>(name.clone().into());
            if path.exists() {
                found_path = Some(path);
                break;
            }
        }

        if let Some(path) = found_path {
            let bytes = fs::read(path)?;
            return self.load_from_bytes(name, &bytes);
        }

        Err(anyhow!("Could not locate classfile {}", name))
    }

    pub fn bootstrap(&mut self) -> Result<(WrappedClassObject, WrappedClassObject)> {
        let meta_class = self.load_class("java/lang/Class".to_string())?;
        self.meta_class_object = Some(meta_class);

        let string_class = self.load_class("java/lang/String".to_string())?;

        Ok((self.meta_class_object.clone().unwrap(), string_class))
    }
}
