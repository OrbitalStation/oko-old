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
        let Expr { value, ty } = Expr::parse(input, &ctx.set_not_primary())?;
        input.close_brace()?;
        Result(Ok((Self {
            value
        }, ty)))
    }
}

#[derive(Clone)]
pub enum PrimitiveExpr <'code> {
    Ident(Spanned <&'code str>),
    Braced(Box <BracedExpr <'code>>),
    // Tuple {
    //     value: Vec <Expr <'code>>,
    //     span: Span
    // }
}

impl <'code> GetSpan for PrimitiveExpr <'code> {
    fn span(&self) -> Span {
        match self {
            Self::Ident(ident) => ident.span,
            Self::Braced(braced) => braced.span(),
            //Self::Tuple { span, .. } => *span
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
                .finish(),
            // Self::Tuple { value, .. } => {
            //     let mut builder = f.debug_tuple("");
            //     for expr in value {
            //         builder.field(&expr.debug(input));
            //     }
            //     builder.finish()
            // }
        }
    }
}

impl <'code> PrimitiveExpr <'code> {
    pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        macro_rules! parse {
            ($input:ident, $( $name:ident => $expr:expr => { $($tt:tt)* } )*) => {
                $(
                    c_like_concat::concat!(
                        let $name ## _cur = $input.get();
                        let $name ## _err = match $expr.0 {
                            Ok(ok) => return Result(Ok({ $( $tt )* })),
                            Err(err) => err
                        };
                        let $name ## _zip = ($name ## _cur, $name ## _err);
                    );
                )*

                let err = [$( c_like_concat::concat!($name ## _zip) ),*].into_iter().max_by_key(|x| x.0).unwrap().1;

                Result(Err(err))
            };
        }

        parse! {
            input,

            ident => input.ident_as_spanned_str() => {
                if let Some(var) = ctx.variables().find(|v| v.name == ok) {
                    (Self::Ident(ok), var.ty.clone())
                } else {
                    return Result(Err(Error {
                        span: ok.span,
                        message: format!("no variable named `{}` found", ok.data),
                        clarifying: String::from("here"),
                        filename: input.filename.to_string(),
                        code: input.code.to_string()
                    }))
                }
            }

            braced => BracedExpr::parse(input, ctx) => {
                (Self::Braced(Box::new(ok.0)), ok.1)
            }
        }
        //
        // if let Result(Ok(ident)) = input.ident_as_spanned_str() {
        //     Result(if let Some(var) = ctx.variables().find(|v| v.name == ident) {
        //         Ok((Self::Ident(ident), var.ty.clone()))
        //     } else {
        //         Err(Error {
        //             span: ident.span,
        //             message: format!("no variable named `{}` found", ident.data),
        //             clarifying: String::from("here"),
        //             filename: input.filename.to_string(),
        //             code: input.code.to_string()
        //         })
        //     })
        // } else if let Some() {
        //     let (braced, ty) = BracedExpr::parse(input, ctx)?;
        //     Result(Ok((Self::Braced(Box::new(braced)), ty)))
        // }
    }
}
