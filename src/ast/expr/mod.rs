use core::fmt::{Debug, Formatter, Result as FmtResult};
use crate::*;

#[derive(Clone, Debug)]
pub struct UnOp <T: Debug> {
    pub value: T,
    pub operator: Span
}

#[derive(Clone, Debug)]
pub struct BinOp <T: Debug> {
    pub left: T,
    pub right: T,
    pub operator: Span
}

#[inline]
fn unspan(token: Result <&Token>) -> Option <Span> {
    token.0.ok().map(|token| token.span)
}

macro_rules! define {
    ($name:ident = $full:ty, $partial:ty) => {
        #[derive(Clone, Debug)]
        pub enum $name <'code> {
            Full(Box <$full>),
            Partial(Box <$partial>)
        }
    };
}

macro_rules! precedence {
    (
        'unop: $( $un_name:ident [$($un_fn_debug:ident $un_fn_enum:ident $un_fn:ident )*] )*
        'binop: $( $bin_name:ident [$($bin_fn_debug:ident $bin_fn_enum:ident $bin_fn:ident )*] )*
    ) => {
        #[derive(Copy, Clone, Eq, PartialEq)]
        #[repr(u8)]
        pub enum UnaryOperator {
            $( $( $un_fn_enum, )* )*
        }

        impl Debug for UnaryOperator {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                f.write_str(match self {$($(
                    Self::$un_fn_enum => stringify!($un_fn_debug),
                )*)*})
            }
        }

        #[derive(Copy, Clone, Eq, PartialEq)]
        #[repr(u8)]
        pub enum BinaryOperator {
            $( $( $bin_fn_enum, )* )*
        }

         impl Debug for BinaryOperator {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                f.write_str(match self {$($(
                    Self::$bin_fn_enum => stringify!($bin_fn_debug),
                )*)*})
            }
        }

        precedence!(@rev(@un) [$( ($un_name { $($un_fn_debug $un_fn_enum $un_fn )* }) )*] | []);
        precedence!(@rev(@bin) [$( ($bin_name { $($bin_fn_debug $bin_fn_enum $bin_fn )* }) )* ($( $un_name )*)] | []);

        precedence!(@expr $( $bin_name )*);
    };

    (@bin ($( $un_name:ident )*) ($name:ident { $( $fun:tt )* }) ) => {
        precedence!(@bin-define $name ($( $un_name )*) $( $fun )*);
    };

    (@bin ($( $un_name:ident )*) ($name:ident { $( $fun:ident )* }) ($next_name:ident { $( $next_fun:tt )* }) $( $tail:tt )*) => {
        precedence!(@bin-define $name ($next_name) $( $fun )*);
        precedence!(@bin ($( $un_name )*) ($next_name { $( $next_fun )* }) $( $tail )*);
    };

    (@bin-define $name:ident ($sub:ident) $( $debug:ident $enumm:ident $fun:ident )*) => {
        define!($name = BinOp <$sub <'code>>, $sub <'code>);

        impl <'code> $name <'code> {
            fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
                let (left, left_ty) = $sub::parse(input, ctx)?;

                Result(Ok(match precedence!(@sign-fun(BinaryOperator) input, $( $fun $enumm )*) {
                    Some((span, operator)) => {
                        let (right, right_ty) = $sub::parse(input, ctx)?;

                        let result_ty = match left_ty.perform_binary_operation(input, operator, &right_ty) {
                            Some(x) => x,
                            None => return Result(Err(Error {
                                span,
                                message: format!("cannot {operator:?} the `{:?}` and `{:?}` types", left_ty.debug(input), right_ty.debug(input)),
                                clarifying: format!("incompatible operator and operands"),
                                filename: input.filename.to_string(),
                                code: input.code.to_string()
                            }))
                        };

                        (Self::Full(Box::new(BinOp {
                            left,
                            right,
                            operator: span
                        })), result_ty)
                    },
                    None => (Self::Partial(Box::new(left)), left_ty)
                }))
            }
        }
    };

    (@bin-define $name:ident ($head:ident $( $tail:ident )+) $( $fun:tt )*) => {
        precedence!(@bin-define $name ($( $tail )+) $( $fun )*);
    };

    (@un ($name:ident { $( $debug:ident $enumm:ident $fun:ident )* })) => {
        define!($name = UnOp <PrimitiveExpr <'code>>, PrimitiveExpr <'code>);

        impl <'code> $name <'code> {
            fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
                Result(Ok(match precedence!(@sign-fun(UnaryOperator) input, $( $fun $enumm )*) {
                    Some((span, operator)) => {
                        let (value, ty) = PrimitiveExpr::parse(input, ctx)?;

                        let result_ty = match ty.perform_unary_operation(input, operator) {
                            Some(x) => x,
                            None => return Result(Err(Error {
                                span,
                                message: format!("cannot {operator:?} the `{:?}` type", ty.debug(input)),
                                clarifying: format!("incompatible operator and operand"),
                                filename: input.filename.to_string(),
                                code: input.code.to_string()
                            }))
                        };

                        (Self::Full(Box::new(UnOp {
                            value,
                            operator: span
                        })), result_ty)
                    },
                    None => {
                        let (operand, ty) = PrimitiveExpr::parse(input, ctx)?;
                        (Self::Partial(Box::new(operand)), ty)
                    }
                }))
            }
        }
    };

    // (@un ($name:ident [$( $fun:ident )*]) $( $tail:tt )+) => {
    // TODO
    // };

    (@sign-fun($prefix:ident) $input:ident, $single:ident $operator:ident) => {
        unspan($input.$single()).map(|x| (x, $prefix::$operator))
    };

    (@sign-fun($prefix:ident) $input:ident, $head:ident $operator:ident $( $tail:ident )+) => {
        precedence!(@sign-fun($prefix) $input, $head $operator).or_else(|| precedence!(@sign-fun($prefix) $input, $( $tail )+))
    };

    (@rev($( $label:tt )*) [] | [$( $rev:tt )*]) => {
        precedence! { $( $label )* $( $rev )* }
    };

    (@rev($( $label:tt )*) [( $( $head:tt )* ) $( ( $( $tail:tt )* ) )*] | [$( $rev:tt )*]) => {
        precedence! { @rev($( $label )*) [$( ( $( $tail )* ) )*] | [( $( $head )* ) $( $rev )*] }
    };

    (@expr $last:ident) => {
        #[derive(Clone)]
        pub struct Expr <'code> {
            pub value: $last <'code>,
            pub ty: TypeIndex
        }

        impl <'code> ParseDebug for Expr <'code> {
            fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
                f.debug_struct("Expr")
                    .field("value", &self.value)
                    .field("ty", &self.ty.debug(input))
                    .finish()
            }
        }

        impl <'code> Expr <'code> {
            pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <Self> {
                let (value, ty) = $last::parse(input, ctx)?;

                Result(Ok(Self {
                    value,
                    ty
                }))
            }
        }
    };

    (@expr $head:ident $( $tail:ident )+) => {
        precedence!(@expr $( $tail )+);
    };
}

precedence! {
    'unop:
        PlusMinusExpr [pos Pos plus negate Neg minus]

    'binop:
        MulDivExpr [multiply Mul star divide Div slash]
        SumDiffExpr [add Add plus sub Sub minus]
}

#[derive(Clone, Debug)]
pub enum PrimitiveExpr <'code> {
    Ident(Spanned <&'code str>)
}

impl <'code> PrimitiveExpr <'code> {
    fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        let ident = input.ident_as_spanned_str()?;

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
    }
}
