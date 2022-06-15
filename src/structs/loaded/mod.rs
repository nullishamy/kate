//! loaded types are constructed from a set of `raw` types.
//! loaded types will resolve all references to other parts of the class file (namely constants)
//! and provide some level of abstraction over the data.

pub mod attribute;
pub mod classfile;
pub mod constant_pool;
pub mod constructors;
pub mod default_attributes;
pub mod field;
pub mod interface;
pub mod method;
pub mod package;

mod classfile_helper;
