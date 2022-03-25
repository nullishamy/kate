use crate::runtime::security::protection_domain::ProtectionDomain;
use crate::runtime::vm::VM;
use crate::structs::loaded::package::Package;
use crate::structs::raw::classfile::RawClassFile;
use crate::LoadedClassFile;
use anyhow::{anyhow, Result};
use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub type ClassLoaderImpl<'a> = Rc<dyn ClassLoader>;

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
    pub data: Vec<u8>,
    pub protection_domain: Option<ProtectionDomain>,
}

pub trait ClassLoader {
    fn parent(&self) -> Option<&ClassLoaderImpl>;

    fn define_class(&mut self, data: &ClassDefinition) -> Result<&LoadedClassFile>;

    fn define_package(&mut self, data: &PackageDefinition) -> Result<Package>;

    fn find_class(&self, internal_name: &str) -> Result<ClassDefinition>;

    fn find_loaded_class(&self, internal_name: &str) -> Option<&LoadedClassFile>;

    fn get_package(&self, internal_name: &str) -> Result<&Package>;

    fn get_packages(&self) -> Result<Vec<&Package>>;

    fn load_class(&mut self, internal_name: &str) -> Result<&LoadedClassFile> {
        // let found = self.find_loaded_class(internal_name);
        //
        // if let Some(found) = found {
        //     return Ok(found);
        // }
        //
        // let parent = self.parent();
        //
        // if let Some(parent) = parent {
        //     return Rc::clone(parent).load_class(internal_name);
        // }
        //
        // return self.define_class(&self.find_class(internal_name)?);
        todo!("storing mutable references in a chain is causing issues here")
    }
}
