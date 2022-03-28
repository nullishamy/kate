use crate::runtime::classload::loader::{
    ClassDefinition, ClassLoader, ClassLoaderImpl, MutableClassLoader, MutatedLoader,
    PackageDefinition,
};
use crate::structs::loaded::package::Package;
use crate::{ClassFileParser, LoadedClassFile};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::rc::Rc;

pub struct SystemClassLoader {
    classes: HashMap<String, Rc<LoadedClassFile>>,
}

impl SystemClassLoader {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }
}

impl MutableClassLoader for SystemClassLoader {
    fn define_class(
        &self,
        data: &ClassDefinition,
    ) -> Result<MutatedLoader<Rc<Self>, Rc<LoadedClassFile>>> {
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

        let res = Rc::new(res);
        let mut old_classes = HashMap::with_capacity(self.classes.len());

        old_classes.clone_from(&self.classes);
        old_classes.insert(name.to_owned(), res);

        let new_loader = SystemClassLoader {
            classes: old_classes,
        };

        let new_loader = &Rc::new(new_loader);
        let _ref = new_loader.classes.get(&name).unwrap();

        Ok((Rc::clone(new_loader), Rc::clone(_ref)))
    }

    fn define_package(
        &self,
        data: &PackageDefinition,
    ) -> Result<MutatedLoader<Rc<Self>, Rc<Package>>> {
        todo!()
    }
}

impl ClassLoader for SystemClassLoader {
    fn parent(&self) -> Option<ClassLoaderImpl> {
        None
    }

    fn find_class(&self, internal_name: &str) -> Result<ClassDefinition> {
        let bytes = ClassFileParser::bytes(internal_name.to_owned())?;
        Ok(ClassDefinition {
            internal_name: Some(internal_name.to_owned()),
            data: bytes,
            protection_domain: None,
        })
    }

    fn find_loaded_class(&self, internal_name: &str) -> Option<Rc<LoadedClassFile>> {
        self.classes.get(internal_name).map(|c| Rc::clone(c))
    }

    fn get_package(&self, internal_name: &str) -> Result<Rc<Package>> {
        todo!()
    }

    fn get_packages(&self) -> Result<Vec<Rc<Package>>> {
        todo!()
    }
}
