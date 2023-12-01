#![doc = include_str!("../README.md")]

mod parser;
mod types;

pub mod prelude {
    pub use crate::parser::*;
    pub use crate::types::*;
}
