use crate::*;
use core::fmt::{Formatter, Write, Result as FmtResult};

#[derive(Clone)]
#[repr(u8)]
pub enum BakedTypeBaseKind <'code> {
    Builtin(usize),
    TypeProduct(Punctuated <'code, TypedVariable <'code>, "\n\t">)
}

impl <'code> ParseDebug for BakedTypeBaseKind <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str("BakedTypeBaseKind::")?;

        match self {
            Self::Builtin(_) => f.write_str("Builtin"),
            Self::TypeProduct(fields) => {
                f.write_str("TypeProduct(")?;
                fields.debug_impl(input, f)?;
                f.write_char(')')
            }
        }?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct BakedTypeBase <'code> {
    pub kind: BakedTypeBaseKind <'code>,
    pub name: Spanned <&'code str>
}

impl <'code> BakedTypeBase <'code> {
    pub const fn builtin(idx: usize, name: &'code str) -> Self {
        Self {
            kind: BakedTypeBaseKind::Builtin(idx),
            name: Spanned {
                data: name,
                span: Span::DEFAULT
            }
        }
    }
}

impl <'code> ParseDebug for BakedTypeBase <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.debug_struct("BakedTypeBase")
            .field("kind", &self.kind.debug(input))
            .field("name", &self.name)
        .finish()
    }
}
