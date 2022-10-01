mod fun;
pub use fun::*;

mod ty;
pub use ty::*;

use core::fmt::{Formatter, Write, Result as FmtResult};
use crate::*;

#[derive(Clone)]
#[repr(u8)]
pub enum Item <'code> {
	Fn(Fn <'code>),
	Ty(RawTypeDefinitionIndex)
}

impl <'code> ParseDebug for Item <'code> {
	fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
		f.write_str("Item(")?;
		match self {
			Self::Fn(fun) => fun.debug_impl(input, f),
			Self::Ty(ty) => ty.debug_impl(input, f)
		}?;
		f.write_char(')')
	}
}

macro_rules! tryok {
    ($v:ident, $result:expr) => {{
        let result = $result;
        match result.0 {
            Ok(ok) => return Result(Ok(Self::$v(ok))),
            Err((parsed, err)) => (parsed, Result(Err(err)))
        }
    }};
}

impl <'code> Parse <'code> for Item <'code> {
	fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
		let ty = tryok!(Ty, RawTypeDefinitionIndex::parse_with_returning_cur(input));
		let fun = tryok!(Fn, Fn::parse_with_returning_cur(input));

		if ty.0 > fun.0 {
			ty.1
		} else {
			fun.1
		}
	}
}
