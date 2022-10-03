#![feature(try_trait_v2)]
#![feature(adt_const_params)]
#![feature(generic_associated_types)]

#![allow(incomplete_features)]

mod token;
pub use token::*;

mod ast;
pub use ast::*;

mod span;
pub use span::*;

mod error;
pub use error::*;

pub const SPACES_IN_TAB: u32 = 4;
