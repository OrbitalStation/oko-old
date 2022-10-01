use crate::*;
use core::fmt::{Formatter, Result as FmtResult};

/// a b c: T
#[derive(Clone)]
pub struct TypedVariablesSet <'code> {
    pub names: Vec <Spanned <&'code str>>,
    pub ty: TypeIndex
}

impl <'code> ParseDebug for TypedVariablesSet <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        print_punctuated_seq::<_, " ">(self.names.iter(), f)?;
        f.write_str(": ")?;
        self.ty.debug_impl(input, f)
    }
}

impl <'code> Parse <'code> for TypedVariablesSet <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let mut names = vec![input.ident_as_spanned_str()?];

        while let Result(Ok(ok)) = input.ident() {
            // SAFETY: ident returns an identifier which has its str
            names.push(unsafe { ok.to_spanned_str().unwrap_unchecked() })
        }

        input.two_dots()?;

        let ty = TypeIndex::parse(input)?;

        Result(Ok(Self {
            names,
            ty
        }))
    }
}
