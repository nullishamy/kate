use crate::parse::classfile::ClassFile;
use crate::runtime::loaded_class::LoadedClass;

struct BootstrapClassLoader<'a> {
    classes: Vec<LoadedClass<'a>>,
}

impl<'a> BootstrapClassLoader<'a> {
    pub fn new() -> Self {
        Self { classes: vec![] }
    }

    pub fn load_class(&self, class_file: ClassFile<'a>) -> LoadedClass {
        LoadedClass::new(class_file)
    }
}
