use crate::{
    attributes::Attributes,
    flags::{ClassFileAccessFlags, FieldAccessFlags, MethodAccessFlags},
    pool::{
        ConstantClass, ConstantEntry, ConstantField, ConstantNameAndType, ConstantPool,
        ConstantUtf8,
    },
};
use anyhow::Result;
use parking_lot::RwLock;
use std::{convert::TryInto, fmt, marker::PhantomData, sync::Arc};
use support::types::MethodDescriptor;

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

impl IntoIterator for Fields {
    type Item = Field;
    type IntoIter = std::vec::IntoIter<Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
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
    pub fn locate(&self, descriptor: &MethodDescriptor) -> Option<&Method> {
        return self.values.iter().find(|v| {
            let d: MethodDescriptor = (v.name.resolve().string(), v.descriptor.resolve().string())
                .try_into()
                .unwrap();

            &d == descriptor
        });
    }
}

#[derive(Debug, Clone)]
pub struct Interfaces {
    pub(crate) values: Vec<Addressed<ConstantClass>>,
}

impl IntoIterator for Interfaces {
    type Item = Addressed<ConstantClass>;
    type IntoIter = std::vec::IntoIter<Addressed<ConstantClass>>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
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
    entries: Arc<RwLock<Vec<ConstantEntry>>>,
}

impl<T> Addressed<T> {
    pub fn from(index: u16, pool: Arc<RwLock<Vec<ConstantEntry>>>) -> Self {
        Self {
            phantom: PhantomData,
            index,
            entries: pool,
        }
    }

    pub fn index(&self) -> u16 {
        self.index
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
                let entries = self.entries.read();
                let value = entries
                    .get((self.index - 1) as usize)
                    .ok_or(anyhow::anyhow!("no value found"))?;

                match value {
                    ConstantEntry::$enum(data) => Ok(data.clone()),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "expected {} got type {:#?} @ {}",
                            stringify!($enum),
                            value,
                            self.index
                        ))
                    }
                }
            }
        }
    };
}

impl Resolvable<ConstantEntry> for Addressed<ConstantEntry> {
    fn try_resolve(&self) -> Result<ConstantEntry> {
        let pool = self.entries.read();
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
