use crate::runtime::security::protection_domain::ProtectionDomain;
use crate::runtime::vm::VM;
use crate::structs::loaded::package::Package;
use crate::structs::raw::classfile::RawClassFile;
use crate::LoadedClassFile;
use anyhow::{anyhow, Result};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

pub struct PackageDefinition {
    pub internal_name: String,
    pub spec_title: String,
    pub spec_version: String,
    pub spec_vendor: String,
    pub impl_title: String,
    pub impl_version: String,
    pub impl_vendor: String,
    pub seal_base: Option<String>,
}

pub struct ClassDefinition {
    pub internal_name: Option<String>,
    pub bytes: Vec<u8>,
    pub protection_domain: Option<ProtectionDomain>,
}

pub trait ClassLoader<T>
where
    T: Sized + ClassLoader<T>,
{
    fn parent(&self) -> Option<Arc<RwLock<T>>>;

    fn find_class(&self, internal_name: &str) -> Result<ClassDefinition>;
    fn find_loaded_class(&self, internal_name: &str) -> Option<Arc<LoadedClassFile>>;
    fn get_package(&self, internal_name: &str) -> Result<Arc<Package>>;
    fn get_packages(&self) -> Result<Vec<Arc<Package>>>;

    fn define_class(&mut self, data: ClassDefinition) -> Result<Arc<LoadedClassFile>>;
    fn define_package(&self, data: PackageDefinition) -> Result<Arc<Package>>;

    fn load_class(&mut self, internal_name: &str) -> Result<Arc<LoadedClassFile>> {
        let found = self.find_loaded_class(internal_name);

        if let Some(found) = found {
            return Ok(found);
        }

        let mut root = self.parent();

        while let Some(parent) = root {
            if let Some(loaded) = parent.read().unwrap().find_loaded_class(internal_name) {
                return Ok(loaded);
            }

            root = parent.read().unwrap().parent();
        }

        self.define_class(self.find_class(internal_name)?)
    }
}
