//! provides the various structs that the class file parser will deserialize to.
//! this also provides types for descriptor parsing, bitflags and generic JVM types.

pub mod bitflag;
pub mod descriptor;
pub mod loaded;
pub mod raw;
pub mod types;

pub type JvmPointer = isize;
