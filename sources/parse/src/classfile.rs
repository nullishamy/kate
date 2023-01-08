use crate::{
    attributes::Attributes,
    flags::{ClassFileAccessFlags, FieldAccessFlags, MethodAccessFlags},
    pool::{
        ConstantClass, ConstantEntry, ConstantField, ConstantNameAndType, ConstantPool,
        ConstantUtf8,
    },
};
use anyhow::Result;
use std::{fmt, marker::PhantomData, rc::Rc, sync::RwLock};

#[derive(Debug, Clone)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
    pub meta_data: MetaData,

    pub access_flags: ClassFileAccessFlags,
    pub this_class: Addressed<ConstantClass>,
    pub super_class: Option<Addressed<ConstantClass>>,

    pub interfaces: Interfaces,
    pub fields: Fields,
    pub methods: Methods,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub flags: FieldAccessFlags,
    pub name: Addressed<ConstantUtf8>,
    pub descriptor: Addressed<ConstantUtf8>,
    pub attributes: Attributes,
}
#[derive(Debug, Clone)]
pub struct Fields {
    pub(crate) values: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub flags: MethodAccessFlags,
    pub name: Addressed<ConstantUtf8>,
    pub descriptor: Addressed<ConstantUtf8>,
    pub attributes: Attributes,
}
#[derive(Debug, Clone)]
pub struct Methods {
    pub(crate) values: Vec<Method>,
}

impl Methods {
    pub fn locate(&self, name: String, descriptor: String) -> Option<&Method> {
        return self.values.iter().find(|v| {
            let mname = v.name.resolve().string();
            let mdescriptor = v.descriptor.resolve().string();
            name == mname && descriptor == mdescriptor
        });
    }
}

#[derive(Debug, Clone)]
pub struct Interfaces {
    pub(crate) values: Vec<Addressed<ConstantClass>>,
}

#[derive(Debug, Clone)]
pub struct MetaData {
    pub minor_version: u16,
    pub major_version: u16,
}
#[derive(Clone)]
pub struct Addressed<T> {
    phantom: PhantomData<T>,

    index: u16,
    entries: Rc<RwLock<Vec<ConstantEntry>>>,
}

impl<T> Addressed<T> {
    pub fn from(index: u16, pool: Rc<RwLock<Vec<ConstantEntry>>>) -> Self {
        Self {
            phantom: PhantomData,
            index,
            entries: pool,
        }
    }
}

impl<T> fmt::Debug for Addressed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Addressed {{ {} }}", self.index)
    }
}

pub trait Resolvable<T> {
    fn resolve(&self) -> T {
        self.try_resolve().unwrap()
    }

    fn try_resolve(&self) -> Result<T>;
}

macro_rules! address {
    ($type: ty, $enum: ident) => {
        impl Resolvable<$type> for Addressed<$type> {
            fn try_resolve(&self) -> anyhow::Result<$type> {
                let entries = self.entries.read().expect("could not lock pool");
                let value = entries
                    .get((self.index - 1) as usize)
                    .ok_or(anyhow::anyhow!("no value found"))?;

                match value {
                    ConstantEntry::$enum(data) => Ok(data.clone()),
                    _ => panic!(
                        "expected {} got type {:#?} @ {}",
                        stringify!($enum),
                        value,
                        self.index
                    ),
                }
            }
        }
    };
}

impl Resolvable<ConstantEntry> for Addressed<ConstantEntry> {
    fn try_resolve(&self) -> Result<ConstantEntry> {
        let pool = self.entries.read().expect("could not lock pool");
        let value = pool
            .get((self.index - 1) as usize)
            .ok_or(anyhow::anyhow!("no value found"))?;

        Ok(value.clone())
    }
}

address!(ConstantClass, Class);
address!(ConstantField, Field);
address!(ConstantNameAndType, NameAndType);
address!(ConstantUtf8, Utf8);
