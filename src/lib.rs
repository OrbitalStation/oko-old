#![feature(try_trait_v2)]
#![feature(adt_const_params)]
#![feature(generic_associated_types)]
#![feature(iter_advance_by)]

#![allow(incomplete_features)]

modules!(token ast span error handlers);

pub const SPACES_IN_TAB: u32 = 4;

#[macro_export]
macro_rules! modules {
    ($( $name:ident )*) => {$(
		mod $name;
		pub use $name::*;
	)*};
}
