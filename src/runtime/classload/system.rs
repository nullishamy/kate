use crate::runtime::classload::loader::{
    ClassDefinition, ClassLoader, ClassLoaderImpl, PackageDefinition,
};
use crate::structs::loaded::package::Package;
use crate::{ClassFileParser, LoadedClassFile};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct SystemClassLoader {
    classes: HashMap<String, LoadedClassFile>,
}

impl SystemClassLoader {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }
}

impl ClassLoader for SystemClassLoader {
    fn parent(&self) -> Option<&ClassLoaderImpl> {
        None
    }

    fn define_class(&mut self, data: &ClassDefinition) -> Result<&LoadedClassFile> {
        let name = data.internal_name.to_owned();

        if name.is_none() {
            return Err(anyhow!(
                "name was not present, and anonymous classes are not supported yet"
            ));
        }

        let name = name.unwrap();

        let res = LoadedClassFile::from_raw(
            ClassFileParser::from_bytes(name.to_string(), data.data.to_owned()).parse()?,
        )?;

        self.classes.insert(name.to_string(), res);

        // TODO: investigate a better way to do this
        Ok(self.classes.get(&*name.to_string()).unwrap())
    }

    fn define_package(&mut self, data: &PackageDefinition) -> Result<Package> {
        unimplemented!()
    }

    fn find_class(&self, internal_name: &str) -> Result<ClassDefinition> {
        let bytes = ClassFileParser::bytes(internal_name.to_owned())?;
        Ok(ClassDefinition {
            internal_name: Some(internal_name.to_owned()),
            data: bytes,
            protection_domain: None,
        })
    }

    fn find_loaded_class(&self, internal_name: &str) -> Option<&LoadedClassFile> {
        self.classes.get(internal_name)
    }

    fn get_package(&self, internal_name: &str) -> Result<&Package> {
        unimplemented!()
    }

    fn get_packages(&self) -> Result<Vec<&Package>> {
        unimplemented!()
    }
}
