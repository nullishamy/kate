#![feature(pointer_byte_offsets)]
#![feature(offset_of)]
#![allow(clippy::new_without_default)]

pub mod vm;
pub mod object;
pub mod native;
pub mod error;