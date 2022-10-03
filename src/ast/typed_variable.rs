use crate::*;
use core::fmt::{Formatter, Result as FmtResult};

#[derive(Clone)]
pub struct TypedVariable <'code> {
    pub name: Spanned <&'code str>,
    pub ty: TypeIndex
}

impl <'code> TypedVariable <'code> {
    pub fn parse(input: &mut ParseInput <'code>) -> Result <Vec <Self>> {
        Result(TypedVariableWrap::parse(input).0.map(|x| x.0))
    }
}

impl <'code> ParseDebug for TypedVariable <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str(&self.name.data)?;
        f.write_str(": ")?;
        self.ty.debug_impl(input, f)
    }
}

struct TypedVariableWrap <'code> (Vec <TypedVariable <'code>>);

impl <'code> Parse <'code> for TypedVariableWrap <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let mut names = vec![input.ident_as_spanned_str()?];

        while let Result(Ok(ok)) = input.ident() {
            // SAFETY: ident returns an identifier which has its str
            names.push(unsafe { ok.to_spanned_str().unwrap_unchecked() })
        }

        input.two_dots()?;

        let ty = TypeIndex::parse(input)?;

        Result(Ok(Self(names.into_iter().map(|name| TypedVariable {
            name,
            ty: ty.clone()
        }).collect())))
    }
}
