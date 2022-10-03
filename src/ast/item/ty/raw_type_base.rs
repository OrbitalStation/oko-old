use crate::*;
use core::fmt::{Formatter, Write, Result as FmtResult};

#[repr(u8)]
#[derive(Clone)]
pub enum RawTypeBase <'code> {
    Stub(Spanned <&'code str>),
    Backed(RawTypeDefinition <'code>)
}

impl <'code> RawTypeBase <'code> {
    pub const fn name(&self) -> Spanned <&'code str> {
        match self {
            Self::Stub(name) => *name,
            Self::Backed(RawTypeDefinition { name, .. }) => *name
        }
    }
}

impl <'code> ParseDebug for RawTypeBase <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        match self {
            Self::Stub(name) => f.write_fmt(format_args!("RawTypeBaseBackedWithADefinition::Stub({name:?})")),
            Self::Backed(def) => {
                f.write_str("RawTypeBaseBackedWithADefinition::Backed(")?;
                def.debug_impl(input, f)?;
                f.write_char(')')
            }
        }
    }
}
