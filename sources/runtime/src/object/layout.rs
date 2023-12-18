use std::{
    alloc::{alloc_zeroed, Layout},
    collections::HashMap,
};

use parking_lot::RwLock;
use parse::{
    attributes::ConstantValueAttribute,
    classfile::{ClassFile, Field, Resolvable},
    flags::FieldAccessFlag,
    pool::ConstantEntry,
};
use support::descriptor::FieldType;

use crate::{error::Throwable, internal, internalise};

use self::types::JavaType;

use super::{builtins::Object, interner::intern_string, value::RuntimeValue};

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

#[derive(Debug)]
pub struct ClassFileLayout {
    pub(crate) layout: Layout,
    pub(crate) field_info: HashMap<String, FieldInfo>,
    pub(crate) statics: RwLock<HashMap<String, FieldInfo>>,
}

impl ClassFileLayout {
    pub fn from_java_type(ty: JavaType) -> Self {
        Self {
            layout: ty.layout,
            field_info: HashMap::new(),
            statics: RwLock::new(HashMap::new()),
        }
    }

    pub fn empty() -> Self {
        Self {
            layout: Layout::new::<()>(),
            field_info: HashMap::new(),
            statics: RwLock::new(HashMap::new()),
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

    pub fn statics(&self) -> &RwLock<HashMap<String, FieldInfo>> {
        &self.statics
    }

    pub fn alloc(&self) -> *mut Object {
        unsafe { alloc_zeroed(self.layout).cast::<Object>() }
    }
}

pub mod types {
    use std::{alloc::Layout, ops::Deref};

    use support::descriptor::{BaseType, FieldType};

    use crate::{
        error::Throwable,
        internal, internalise,
        object::{
            builtins::{Array, Object},
            mem::RefTo,
        },
    };

    #[derive(Debug, Clone, Copy)]
    pub struct JavaType {
        pub alignment: Alignment,
        pub size: Size,
        pub layout: Layout,
        pub name: &'static str,
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
        pub const fn new<T>(name: &'static str) -> Self {
            let layout = Layout::new::<T>();
            Self {
                alignment: Alignment(layout.align()),
                size: Size(layout.size()),
                name,
                layout,
            }
        }

        pub const fn from_layout(layout: Layout, name: &'static str) -> Self {
            Self {
                alignment: Alignment(layout.align()),
                size: Size(layout.size()),
                name,
                layout,
            }
        }
    }

    pub const BOOL: JavaType = JavaType::new::<Bool>("Z");

    pub const CHAR: JavaType = JavaType::new::<Char>("C");
    pub const FLOAT: JavaType = JavaType::new::<Float>("F");
    pub const DOUBLE: JavaType = JavaType::new::<Double>("D");
    pub const BYTE: JavaType = JavaType::new::<Byte>("B");
    pub const SHORT: JavaType = JavaType::new::<Short>("S");
    pub const LONG: JavaType = JavaType::new::<Long>("J");
    pub const INT: JavaType = JavaType::new::<Int>("I");

    pub const OBJECT: JavaType = JavaType::new::<RefTo<Object>>("Object");
    pub const ARRAY_BASE: JavaType = JavaType::new::<RefTo<Array<()>>>("Array");

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
            FieldType::Array(array_ty) => {
                let component = match *array_ty.field_type {
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
                )
                .map_err(internalise!())?;

                JavaType::from_layout(layout, "Runtime evaluated type")
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
            // Static fields are initialised before clinit is ran
            
            // Try and load from the constant value attribute, falling back to descriptor defaults if no attr exists
            let attr = field
                .attributes
                .known_attribute::<ConstantValueAttribute>(&class_file.constant_pool);

            let static_value = if let Ok(entry) = attr {
                let entry = entry.value.resolve();

                match entry {
                    ConstantEntry::String(data) => {
                        let str = data.string();
                        RuntimeValue::Object(intern_string(str)?.erase())
                    }
                    ConstantEntry::Integer(data) => {
                        RuntimeValue::Integral((data.bytes as i32).into())
                    },
                    ConstantEntry::Float(data) => {
                        RuntimeValue::Floating(data.bytes.into())
                    },
                    ConstantEntry::Long(data) => {
                        RuntimeValue::Integral((data.bytes as i64).into())
                    },
                    ConstantEntry::Double(data) => {
                        RuntimeValue::Floating(data.bytes.into())
                    },
                    e => return Err(internal!("cannot use {:#?} in constant value", e)),
                }
            } else {
                let descriptor = &FieldType::parse(field.descriptor.resolve().string())?;
                RuntimeValue::default_for_field(descriptor)
            };

            static_fields.push(FieldInfo {
                name: field.name.resolve().string(),
                value: Some(static_value),
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

    let mut final_layout =
        Layout::from_size_align(base_layout.size(), alignment).map_err(internalise!())?;
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

pub fn full_layout(
    class_file: &ClassFile,
    base_layout: Layout,
) -> Result<ClassFileLayout, Throwable> {
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
        statics: RwLock::new(
            basic_layout
                .statics
                .into_iter()
                .map(|s| (s.name.clone(), s))
                .collect(),
        ),
    })
}
