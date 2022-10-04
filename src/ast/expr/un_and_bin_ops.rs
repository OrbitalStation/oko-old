use core::fmt::{Debug, Formatter, Result as FmtResult};
use crate::*;

#[derive(Clone)]
pub struct UnOp <T> {
    pub value: T,
    pub operator: Span
}

impl <T: GetSpan> GetSpan for UnOp <T> {
    fn span(&self) -> Span {
        Span {
            start: self.operator.start,
            end: self.value.span().end
        }
    }
}

impl <T: ParseDebug> ParseDebug for UnOp <T> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("UnnOp")
            .field("left", &self.value.debug(input))
            .field("operator", &self.operator.get_spanned_lines(&input.code)[0])
            .finish()
    }
}

#[derive(Clone)]
pub struct BinOp <T> {
    pub left: T,
    pub right: T,
    pub operator: Span
}

impl <T: GetSpan> GetSpan for BinOp <T> {
    fn span(&self) -> Span {
        Span {
            start: self.left.span().start,
            end: self.right.span().end
        }
    }
}

impl <T: ParseDebug> ParseDebug for BinOp <T> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("BinOp")
            .field("left", &self.left.debug(input))
            .field("right", &self.right.debug(input))
            .field("operator", &self.operator.get_spanned_lines(&input.code)[0])
            .finish()
    }
}

#[inline]
fn unspan(token: Result <&Token>) -> Option <Span> {
    token.0.ok().map(|token| token.span)
}

macro_rules! precedence {
    (
        NEXT = $next:ident,
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

        precedence!(@rev(@un) [$( ($un_name { $($un_fn_debug $un_fn_enum $un_fn )* }) )* ($next)] | []);
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
        define_expr!($name = BinOp <$sub <'code>>, $sub <'code>);

        impl <'code> $name <'code> {
            pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
                let (left, left_ty) = $sub::parse(input, ctx)?;

                let saved_cur_token_for_possible_early_return_triggered_on_inequality_of_spaces_around_the_operator = input.get();

                Result(Ok(match precedence!(@sign-fun(BinaryOperator) input, $( $fun $enumm )*) {
                    Some((span, operator)) => {
                        let (right, right_ty) = $sub::parse(input, ctx)?;

                        // Remove ugly operators format
                        let are_left_and_operator_close = (span.start.column - left.span().end.column) == 1;
                        let are_right_and_operator_close = (right.span().start.column - span.end.column) == 1;
                        if are_left_and_operator_close != are_right_and_operator_close {
                            input.set(saved_cur_token_for_possible_early_return_triggered_on_inequality_of_spaces_around_the_operator);
                            return Result(Ok((Self::Partial(Box::new(left)), left_ty)))
                        }

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

    (@un ($next:ident) ($name:ident { $( $debug:ident $enumm:ident $fun:ident )* })) => {
        define_expr!($name = UnOp <$next <'code>>, $next <'code>);

        impl <'code> $name <'code> {
            pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
                Result(Ok(match precedence!(@sign-fun(UnaryOperator) input, $( $fun $enumm )*) {
                    Some((span, operator)) => {
                        let (value, ty) = $next::parse(input, ctx)?;

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
                        let (operand, ty) = $next::parse(input, ctx)?;
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
        pub(crate) type LastExpr <'code> = $last <'code>;

        #[derive(Clone)]
        pub struct Expr <'code> {
            pub value: LastExpr <'code>,
            pub ty: TypeIndex
        }

        impl <'code> GetSpan for Expr <'code> {
            fn span(&self) -> Span {
                self.value.span()
            }
        }

        impl <'code> ParseDebug for Expr <'code> {
            fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
                f.debug_struct("Expr")
                    .field("value", &self.value.debug(input))
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
    NEXT = CallExpr,

    'unop:
        PlusMinusExpr [pos Pos plus negate Neg minus]

    'binop:
        MulDivExpr [multiply Mul star divide Div slash]
        SumDiffExpr [add Add plus sub Sub minus]
}
