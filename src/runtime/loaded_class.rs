use crate::parse::classfile::ClassFile;

pub struct LoadedClass<'a> {
    class_file: ClassFile<'a>,
}

impl<'a> LoadedClass<'a> {
    pub fn new(class_file: ClassFile<'a>) -> Self {
        Self { class_file }
    }
}
