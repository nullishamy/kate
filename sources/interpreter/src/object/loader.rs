use std::{alloc::Layout, collections::HashMap, fs, path::PathBuf};

use crate::{object::builtins::{ArrayPrimitive, ArrayType}, error::Throwable, internal, internalise};

use super::{
    builtins::{Class, Object},
    layout::{full_layout, types, ClassFileLayout},
    mem::{HasObjectHeader, RefTo},
};
use parse::{classfile::Resolvable, parser::Parser};
use tracing::debug;

pub fn base_layout() -> Layout {
    Layout::new::<Object>()
}

pub struct ClassLoader {
    class_path: Vec<PathBuf>,
    classes: HashMap<String, RefTo<Class>>,
    meta_class: RefTo<Class>,
}

pub struct BootstrappedClasses {
    pub java_lang_class: RefTo<Class>,
    pub java_lang_object: RefTo<Class>,
    pub java_lang_string: RefTo<Class>,
}

impl ClassLoader {
    pub fn new() -> Self {
        Self {
            class_path: vec![],
            classes: HashMap::new(),
            meta_class: RefTo::null(),
        }
    }

    pub fn for_bytes(&mut self, name_key: String, bytes: &[u8]) -> Result<RefTo<Class>, Throwable> {
        let mut parser = Parser::new(bytes);
        let class_file = parser.parse()?;

        let mut super_class: Option<RefTo<Class>> = None;
        if let Some(ref cls) = class_file.super_class {
            let super_class_name = cls.resolve().name.resolve().string();
            super_class = Some(self.for_name(super_class_name)?);
        }

        // Layout for the actual class we are loading
        let mut layout = full_layout(&class_file, base_layout())?;

        // Add all the superclass fields after our own
        {
            let mut _super = super_class.clone();
            while let Some(sup) = &_super {
                // Get the superclass layout
                let super_classfile = sup.to_ref().class_file();

                // We need to construct our own instead of just using the one stored by the class
                // because it includes the header, which is not relevant for inherited fields.
                let mut super_layout = full_layout(super_classfile, Layout::new::<()>())?;

                // Extend our layout based on it
                let (mut our_new_layout, offset_from_base) =
                    layout.layout().extend(super_layout.layout()).map_err(internalise!())?;

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

                let next_super = sup.to_ref().super_class();
                if !next_super.is_null() {
                    _super = Some(next_super);
                } else {
                    _super = None;
                }
            }
        }

        let cls = Class::new(
            Object::new(self.meta_class.clone(), super_class.unwrap_or(RefTo::null())),
            name_key.clone(),
            class_file,
            layout,
        );

        let object = RefTo::new(cls);

        self.classes.insert(name_key, object.clone());

        Ok(object)
    }

    pub fn for_name(&mut self, name: String) -> Result<RefTo<Class>, Throwable> {
        let formatted_name = format!("{}.class", name);

        if let Some(object) = self.classes.get(&name) {
            debug!("Fast path: {} ({})", name, formatted_name);
            return Ok(object.clone());
        }

        debug!("Slow path: {} ({})", &name, &formatted_name);

        let found_path = self.resolve_name(formatted_name.clone());
        if let Some(path) = found_path {
            let bytes = fs::read(path).map_err(internalise!())?;
            return self.for_bytes(name, &bytes);
        }

        Err(internal!("Could not locate classfile {}", formatted_name))
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

    pub fn classes(&self) -> &HashMap<String, RefTo<Class>> {
        &self.classes
    }

    pub fn add_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.class_path.push(path.into());
        self
    }

    pub fn bootstrap(&mut self) -> Result<BootstrappedClasses, Throwable> {
        let jlc = self.for_name("java/lang/Class".to_string())?;
        self.meta_class = jlc.clone();

        let jlo = self.for_name("java/lang/Object".to_string())?;
        let jls = self.for_name("java/lang/String".to_string())?;

        macro_rules! primitive {
            ($layout_ty: ident, $array_ty: ident,  $name: expr) => {{
                let prim = RefTo::new(Class::new_primitive(
                    Object::new(jlc.clone(), jlo.clone()),
                    $name.to_string(),
                    ClassFileLayout::from_java_type(types::$layout_ty)
                ));
                let array = RefTo::new(Class::new_array(
                    Object::new(prim.clone(), RefTo::null()),
                    format!("[{}", $name.to_string()),
                    ArrayType::Primitive(ArrayPrimitive::$array_ty),
                    ClassFileLayout::from_java_type(types::$layout_ty)
                ));

                (prim, array)
            }};
        }

        macro_rules! insert {
            ($self: expr, $tup: expr, $ty: expr) => {
                self.classes.insert($ty.to_string(), $tup.0);
                self.classes.insert(format!("[{}", $ty.to_string()), $tup.1);
            };
        }

        // Primitives
        let bool = primitive!(BOOL, Bool, "Z");
        insert!(self, bool, "Z");

        let byte = primitive!(BYTE, Byte, "B");
        insert!(self, byte, "B");

        let short = primitive!(SHORT, Short, "S");
        insert!(self, short, "S");

        let char = primitive!(CHAR, Char, "C");
        insert!(self, char.clone(), "C");
        let char_2d_array = RefTo::new(Class::new_array(
            Object::new(char.0.clone(), RefTo::null()),
            "[[C".to_string(),
            ArrayType::Primitive(ArrayPrimitive::Char),
            ClassFileLayout::from_java_type(types::CHAR)
        ));

        self.classes.insert("[[C".to_string(), char_2d_array);

        let int = primitive!(INT, Int, "I");
        insert!(self, int, "I");

        let long = primitive!(LONG, Long, "J");
        insert!(self, long, "J");

        let float = primitive!(FLOAT, Float, "F");
        insert!(self, float, "F");

        let double = primitive!(DOUBLE, Double, "D");
        insert!(self, double, "D");

        let jlo_array = RefTo::new(Class::new_array(
            Object::new(jlo.clone(), RefTo::null()),
            "[Ljava/lang/Object;".to_string(),
            ArrayType::Object(jlo.clone()),
            ClassFileLayout::from_java_type(types::ARRAY_BASE)
        ));

        self.classes.insert("[Ljava/lang/Object;".to_string(), jlo_array);
        
        {
            let cls = self.for_name("java/util/concurrent/ConcurrentHashMap$Segment".to_string())?;
            let arr = RefTo::new(Class::new_array(
                Object::new(cls.clone(), cls.to_ref().super_class()),
                "[Ljava/util/concurrent/ConcurrentHashMap$Segment;".to_string(),
                ArrayType::Object(jlo.clone()),
                ClassFileLayout::from_java_type(types::ARRAY_BASE)
            ));
    
            self.classes.insert("[Ljava/util/concurrent/ConcurrentHashMap$Segment;".to_string(), arr);
        }

        {
            let cls = self.for_name("java/util/concurrent/ConcurrentHashMap$Node".to_string())?;
            let arr = RefTo::new(Class::new_array(
                Object::new(cls.clone(), cls.to_ref().super_class()),
                "[Ljava/util/concurrent/ConcurrentHashMap$Node;".to_string(),
                ArrayType::Object(jlo.clone()),
                ClassFileLayout::from_java_type(types::ARRAY_BASE)
            ));
    
            self.classes.insert("[Ljava/util/concurrent/ConcurrentHashMap$Node;".to_string(), arr);
        }

        {
            let cls = jls.clone();
            let arr = RefTo::new(Class::new_array(
                Object::new(cls.clone(), cls.to_ref().super_class()),
                "[Ljava/lang/String;".to_string(),
                ArrayType::Object(jlo.clone()),
                ClassFileLayout::from_java_type(types::ARRAY_BASE)
            ));
    
            self.classes.insert("[Ljava/lang/String;".to_string(), arr);
        }
        

        self.classes.iter_mut().for_each(|(_, value)| {
            value.borrow_mut().header_mut().class = self.meta_class.clone();
        });

        Ok(BootstrappedClasses {
            java_lang_class: jlc,
            java_lang_object: jlo,
            java_lang_string: jls,
        })
    }
}
