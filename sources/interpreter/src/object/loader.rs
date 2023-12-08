use std::{alloc::Layout, collections::HashMap, fs, path::PathBuf};

use crate::{error::Throwable, internal, internalise, object::builtins::ArrayPrimitive};

use super::{
    builtins::{Class, Object},
    layout::{full_layout, types, ClassFileLayout},
    mem::{HasObjectHeader, RefTo},
};
use parse::{classfile::Resolvable, parser::Parser};
use support::descriptor::{BaseType, FieldType};
use tracing::debug;

pub fn base_layout() -> Layout {
    Layout::new::<Object>()
}

pub struct ClassLoader {
    class_path: Vec<PathBuf>,
    classes: HashMap<FieldType, RefTo<Class>>,
    meta_class: RefTo<Class>,
}

pub struct BootstrappedClasses {
    pub java_lang_class: RefTo<Class>,
    pub java_lang_object: RefTo<Class>,
    pub java_lang_string: RefTo<Class>,
    pub byte_array_ty: RefTo<Class>
}

impl ClassLoader {
    pub fn new() -> Self {
        Self {
            class_path: vec![],
            classes: HashMap::new(),
            meta_class: RefTo::null(),
        }
    }

    pub fn for_bytes(
        &mut self,
        field_type: FieldType,
        bytes: &[u8],
    ) -> Result<RefTo<Class>, Throwable> {
        let mut parser = Parser::new(bytes);
        let class_file = parser.parse()?;

        let mut super_class: Option<RefTo<Class>> = None;
        if let Some(ref cls) = class_file.super_class {
            let super_class_name = cls.resolve().name.resolve().string();
            let super_class_name = FieldType::parse(format!("L{};", super_class_name))?;
            super_class = Some(self.for_name(super_class_name)?);
        }

        // Layout for the actual class we are loading
        let mut layout = full_layout(&class_file, base_layout())?;

        // Add all the superclass fields after our own
        {
            let mut _super = super_class.clone();
            while let Some(sup) = &_super {
                // Get the superclass layout
                let super_classfile = sup.unwrap_ref().class_file();

                // We need to construct our own instead of just using the one stored by the class
                // because it includes the header, which is not relevant for inherited fields.
                let mut super_layout = full_layout(super_classfile, Layout::new::<()>())?;

                // Extend our layout based on it
                let (mut our_new_layout, offset_from_base) = layout
                    .layout()
                    .extend(super_layout.layout())
                    .map_err(internalise!())?;

                // Align the new layout
                our_new_layout = our_new_layout.pad_to_align();

                // Adjust the offset of the superclass fields to be based on the new offsets
                super_layout.field_info.iter_mut().for_each(|(_, field)| {
                    field.location.offset += offset_from_base;
                });

                // Push superclass fields into our layout
                layout.field_info.extend(super_layout.field_info);

                // Assign our layout to the newly computed one
                layout.layout = our_new_layout;

                let next_super = sup.unwrap_ref().super_class();
                if !next_super.is_null() {
                    _super = Some(next_super);
                } else {
                    _super = None;
                }
            }
        }

        let name = field_type.name();

        let cls = Class::new(
            Object::new(
                self.meta_class.clone(),
                super_class.unwrap_or(RefTo::null()),
            ),
            name,
            class_file,
            layout,
        );

        let object = RefTo::new(cls);

        self.classes.insert(field_type, object.clone());

        Ok(object)
    }

    pub fn for_name(&mut self, field_type: FieldType) -> Result<RefTo<Class>, Throwable> {
        if let Some(object) = self.classes.get(&field_type) {
            debug!("Fast path: {}", field_type.name());
            return Ok(object.clone());
        }

        if let Some(array) = field_type.as_array() {
            let component_ty = self.for_name(*array.field_type.clone())?;
            let cls = Class::new_array(
                Object::new(RefTo::null(), RefTo::null()),
                component_ty,
                ClassFileLayout::from_java_type(types::ARRAY_BASE),
            );

            let cls = RefTo::new(cls);
            self.classes.insert(field_type, cls.clone());
            return Ok(cls);
        }

        let formatted_name = format!("{}.class", field_type.name());
        debug!("Slow path: {}", &formatted_name);

        let found_path = self.resolve_name(formatted_name.clone());
        if let Some(path) = found_path {
            let bytes = fs::read(path).map_err(internalise!())?;
            return self.for_bytes(field_type, &bytes);
        }

        Err(internal!("Could not locate classfile {} ({:#?})", formatted_name, field_type))
    }

    fn resolve_name(&self, name: String) -> Option<PathBuf> {
        let mut found_path: Option<PathBuf> = None;

        for root in self.class_path.iter() {
            let path = root.join::<PathBuf>(name.clone().into());
            if path.exists() {
                found_path = Some(path);
                break;
            }
        }

        found_path
    }

    pub fn classes(&self) -> &HashMap<FieldType, RefTo<Class>> {
        &self.classes
    }

    pub fn add_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.class_path.push(path.into());
        self
    }

    pub fn bootstrap(&mut self) -> Result<BootstrappedClasses, Throwable> {
        let jlc = self.for_name("Ljava/lang/Class;".into())?;
        self.meta_class = jlc.clone();

        let jlo = self.for_name("Ljava/lang/Object;".into())?;
        let jls = self.for_name("Ljava/lang/String;".into())?;

        macro_rules! primitive {
            ($layout_ty: ident, $name: expr) => {{
                let prim = RefTo::new(Class::new_primitive(
                    Object::new(jlc.clone(), jlo.clone()),
                    $name.to_string(),
                    ClassFileLayout::from_java_type(types::$layout_ty),
                ));

                let array = RefTo::new(Class::new_array(
                    Object::new(jlc.clone(), jlo.clone()),
                    prim.clone(),
                    ClassFileLayout::from_java_type(types::ARRAY_BASE),
                ));

                (prim, array)
            }};
        }

        macro_rules! insert {
            ($tup: expr) => {
                self.classes
                    .insert($tup.0.unwrap_ref().name().clone().into(), $tup.0);
                self.classes
                    .insert($tup.1.unwrap_ref().name().clone().into(), $tup.1);
            };
        }


        // Primitives
        let byte = primitive!(BYTE, "B");
        insert!(byte.clone());
        
        insert!(primitive!(BOOL, "Z"));
        insert!(primitive!(SHORT, "S"));
        insert!(primitive!(CHAR, "C"));
        insert!(primitive!(INT, "I"));
        insert!(primitive!(LONG, "J"));
        insert!(primitive!(FLOAT, "F"));
        insert!(primitive!(DOUBLE, "D"));

        self.classes.iter_mut().for_each(|(_, value)| {
            value.unwrap_mut().header_mut().class = self.meta_class.clone();
        });

        Ok(BootstrappedClasses {
            java_lang_class: jlc,
            java_lang_object: jlo,
            java_lang_string: jls,
            byte_array_ty: byte.1
        })
    }
}
