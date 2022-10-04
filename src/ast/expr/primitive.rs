use core::fmt::{Formatter, Result as FmtResult};
use crate::*;

#[derive(Clone)]
pub struct BracedExpr <'code> {
    pub value: LastExpr <'code>
}

impl <'code> GetSpan for BracedExpr <'code> {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl <'code> ParseDebug for BracedExpr <'code> {
    #[inline(always)]
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        self.value.debug_impl(input, f)
    }
}

impl <'code> BracedExpr <'code> {
    fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        input.open_brace()?;
        let Expr { value, ty } = Expr::parse(input, ctx)?;
        input.close_brace()?;
        Result(Ok((Self {
            value
        }, ty)))
    }
}

#[derive(Clone)]
pub enum PrimitiveExpr <'code> {
    Ident(Spanned <&'code str>),
    Braced(Box <BracedExpr <'code>>)
}

impl <'code> GetSpan for PrimitiveExpr <'code> {
    fn span(&self) -> Span {
        match self {
            Self::Ident(ident) => ident.span,
            Self::Braced(braced) => braced.span()
        }
    }
}

impl <'code> ParseDebug for PrimitiveExpr <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str("PrimitiveExpr::")?;
        match self {
            Self::Ident(ident) => f.debug_tuple("Ident")
                .field(ident)
                .finish(),
            Self::Braced(expr) => f.debug_tuple("Braced")
                .field(&expr.debug(input))
                .finish()
        }
    }
}

impl <'code> PrimitiveExpr <'code> {
    pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        if let Result(Ok(ident)) = input.ident_as_spanned_str() {
            Result(if let Some(var) = ctx.variables().find(|v| v.name == ident) {
                Ok((Self::Ident(ident), var.ty.clone()))
            } else {
                Err(Error {
                    span: ident.span,
                    message: format!("no variable named `{}` found", ident.data),
                    clarifying: String::from("here"),
                    filename: input.filename.to_string(),
                    code: input.code.to_string()
                })
            })
        } else {
            let (braced, ty) = BracedExpr::parse(input, ctx)?;
            Result(Ok((Self::Braced(Box::new(braced)), ty)))
        }
    }
}
