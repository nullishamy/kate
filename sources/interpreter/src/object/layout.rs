use std::{
    alloc::{alloc_zeroed, Layout},
    collections::HashMap,
};

use parse::{
    classfile::{ClassFile, Field, Resolvable},
    flags::FieldAccessFlag,
};
use support::descriptor::FieldType;

use crate::{error::Throwable, internalise};

use self::types::JavaType;

use super::{builtins::Object, runtime::RuntimeValue};

#[derive(Debug, Clone)]
/// More expensive to clone, but more comprehensive field information.
/// Used to give out the locations of fields.
pub struct FieldInfo {
    pub name: String,
    pub data: Field,
    pub location: FieldLocation,
    pub value: Option<RuntimeValue>,
}

#[derive(Debug, Clone, Copy)]
/// A trivially copyable location for a field within a class
/// Used by FieldRefTo to access the underlying data from a *const Object.
pub struct FieldLocation {
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct ClassFileLayout {
    pub(crate) layout: Layout,
    pub(crate) field_info: HashMap<String, FieldInfo>,
    pub(crate) statics: HashMap<String, FieldInfo>,
}

impl ClassFileLayout {
    pub fn from_java_type(ty: JavaType) -> Self {
        Self {
            layout: ty.layout,
            field_info: HashMap::new(),
            statics: HashMap::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            layout: Layout::new::<()>(),
            field_info: HashMap::new(),
            statics: HashMap::new(),
        }
    }

    pub fn fields(&self) -> &HashMap<String, FieldInfo> {
        &self.field_info
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn field_info(&self, name: &String) -> Option<&FieldInfo> {
        self.field_info.get(name)
    }

    pub fn field_info_mut(&mut self, name: &String) -> Option<&mut FieldInfo> {
        self.field_info.get_mut(name)
    }

    pub fn static_field_info(&self, name: &String) -> Option<&FieldInfo> {
        self.statics.get(name)
    }

    pub fn static_field_info_mut(&mut self, name: &String) -> Option<&mut FieldInfo> {
        self.statics.get_mut(name)
    }

    pub fn alloc(&self) -> *mut Object {
        unsafe { alloc_zeroed(self.layout).cast::<Object>() }
    }
}

pub mod types {
    use std::{alloc::Layout, ops::Deref};

    use support::descriptor::{BaseType, FieldType};

    use crate::{object::{
        builtins::{Array, Object},
        mem::RefTo,
    }, error::Throwable, internal, internalise};

    #[derive(Debug, Clone, Copy)]
    pub struct JavaType {
        pub alignment: Alignment,
        pub size: Size,
        pub layout: Layout,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Alignment(usize);
    impl Deref for Alignment {
        type Target = usize;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Size(usize);
    impl Deref for Size {
        type Target = usize;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl JavaType {
        pub const fn new(layout: Layout) -> Self {
            Self {
                alignment: Alignment(layout.align()),
                size: Size(layout.size()),
                layout,
            }
        }
    }

    // Bools are described as ints
    pub const BOOL: JavaType = INT;

    pub const CHAR: JavaType = JavaType::new(Layout::new::<Char>());
    pub const FLOAT: JavaType = JavaType::new(Layout::new::<Float>());
    pub const DOUBLE: JavaType = JavaType::new(Layout::new::<Double>());
    pub const BYTE: JavaType = JavaType::new(Layout::new::<Byte>());
    pub const SHORT: JavaType = JavaType::new(Layout::new::<Short>());
    pub const LONG: JavaType = JavaType::new(Layout::new::<Long>());
    pub const INT: JavaType = JavaType::new(Layout::new::<Int>());

    pub const OBJECT: JavaType = JavaType::new(Layout::new::<RefTo<Object>>());
    pub const ARRAY_BASE: JavaType = JavaType::new(Layout::new::<RefTo<Array<()>>>());

    pub type Bool = Int;
    pub type Char = u16;
    pub type Float = f32;
    pub type Double = f64;
    pub type Byte = u8;
    pub type Short = i16;
    pub type Long = i64;
    pub type Int = i32;

    pub fn for_field_type(ty: FieldType) -> Result<JavaType, Throwable> {
        Ok(match ty {
            FieldType::Base(base) => match base {
                BaseType::Boolean => BOOL,
                BaseType::Char => CHAR,
                BaseType::Float => FLOAT,
                BaseType::Double => DOUBLE,
                BaseType::Byte => BYTE,
                BaseType::Short => SHORT,
                BaseType::Int => INT,
                BaseType::Long => LONG,
                BaseType::Void => return Err(internal!("void is not supported")),
            },
            FieldType::Array(ty) => {
                let component = match *ty.field_type {
                    FieldType::Base(ty) => for_field_type(FieldType::Base(ty))?,
                    FieldType::Object(_) => OBJECT,
                    FieldType::Array(_) => todo!(
                        "not sure how to describe nested arrays because Array is not fully implemented"
                    ),
                };

                // We can only allocate the struct and nothing else (such as the elements)
                // The runtime has to re-allocate the array if we want to grow it to a real size
                let layout = Layout::from_size_align(
                    // So, we allocate only the base struct (this assumes the "value" is a zero sized type)
                    // which should be semantically identical to zero values
                    *ARRAY_BASE.size,
                    // Select the largest alignment out of the component or the values in the array struct itself,
                    // this is probably always going to be 8, but we should check anyways.
                    *component.alignment.max(ARRAY_BASE.alignment),
                ).map_err(internalise!())?;

                JavaType::new(layout)
            }
            FieldType::Object(_) => OBJECT,
        })
    }
}

#[derive(Debug)]
pub struct BasicLayout {
    layout: Layout,
    fields: Vec<Field>,
    names: Vec<String>,
    offsets: Vec<usize>,
    statics: Vec<FieldInfo>,
}

pub fn basic_layout(class_file: &ClassFile, base_layout: Layout) -> Result<BasicLayout, Throwable> {
    // Handle statics differently so that they are not included in the layout
    let mut instance_fields = vec![];
    let mut static_fields = vec![];

    for field in class_file.fields.clone().values.into_iter() {
        if field.flags.has(FieldAccessFlag::STATIC) {
            static_fields.push(FieldInfo {
                name: field.name.resolve().string(),
                value: Some(RuntimeValue::default_for_field(&FieldType::parse(
                    field.descriptor.resolve().string(),
                )?)),
                data: field,
                location: FieldLocation { offset: 0 },
            });
        } else {
            instance_fields.push(field);
        }
    }

    let descriptors = instance_fields
        .iter()
        .map(|f| {
            let desc = f.descriptor.resolve().string();
            FieldType::parse(desc).map_err(internalise!())
        })
        .collect::<Result<Vec<_>, Throwable>>()?;

    let names = instance_fields
        .iter()
        .map(|f| f.name.resolve().string())
        .collect::<Vec<_>>();

    let field_layouts = descriptors
        .into_iter()
        .map(|desc| {
            let ty = types::for_field_type(desc)?;
            Layout::from_size_align(*ty.size, *ty.alignment).map_err(internalise!())
        })
        .collect::<Result<Vec<_>, Throwable>>()?;

    // Find the overall alignment for the struct
    let alignment = field_layouts
        .iter()
        .map(|d| d.align())
        .chain(vec![base_layout.align()])
        .max()
        .unwrap(); // Iterator can't be empty because we chain with at least one value

    let mut final_layout = Layout::from_size_align(base_layout.size(), alignment).map_err(internalise!())?;
    let mut offsets = Vec::new();

    for layout in field_layouts {
        let (new_layout, offset) = final_layout.extend(layout).map_err(internalise!())?;
        final_layout = new_layout;
        offsets.push(offset);
    }

    Ok(BasicLayout {
        layout: final_layout.pad_to_align(),
        fields: instance_fields,
        names,
        offsets,
        statics: static_fields,
    })
}

pub fn full_layout(class_file: &ClassFile, base_layout: Layout) -> Result<ClassFileLayout, Throwable> {
    let basic_layout = basic_layout(class_file, base_layout)?;

    let field_info = basic_layout
        .fields
        .iter()
        .zip(basic_layout.offsets)
        .zip(basic_layout.names)
        .map(|((field, offset), name)| {
            (
                name.clone(),
                FieldInfo {
                    name,
                    data: field.clone(),
                    location: FieldLocation { offset },
                    value: None,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    Ok(ClassFileLayout {
        layout: basic_layout.layout,
        field_info,
        statics: basic_layout
            .statics
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect(),
    })
}
