use core::fmt::Debug;
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
        'unop: $( $un_name:ident [$( $un_fn:ident )*] )*
        'binop: $( $bin_name:ident [$( $bin_fn:ident )*] )*
    ) => {
        precedence!(@rev(@un) [$( ($un_name { $( $un_fn )* }) )*] | []);
        precedence!(@rev(@bin) [$( ($bin_name { $( $bin_fn )* }) )* ($( $un_name )*)] | []);

        precedence!(@expr $( $bin_name )*);
    };

    (@bin ($( $un_name:ident )*) ($name:ident { $( $fun:tt )* }) ) => {
        precedence!(@bin-define $name ($( $un_name )*) $( $fun )*);
    };

    (@bin ($( $un_name:ident )*) ($name:ident { $( $fun:ident )* }) ($next_name:ident { $( $next_fun:tt )* }) $( $tail:tt )*) => {
        precedence!(@bin-define $name ($next_name) $( $fun )*);
        precedence!(@bin ($( $un_name )*) ($next_name { $( $next_fun )* }) $( $tail )*);
    };

    (@bin-define $name:ident ($sub:ident) $( $fun:tt )*) => {
        define!($name = BinOp <$sub <'code>>, $sub <'code>);

        impl <'code> Parse <'code> for $name <'code> {
            fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
                let left = $sub::parse(input)?;

                Result(Ok(match precedence!(@sign-fun input, $( $fun )*) {
                    Some(operator) => {
                        let right = $sub::parse(input)?;

                        Self::Full(Box::new(BinOp {
                            left,
                            right,
                            operator
                        }))
                    },
                    None => Self::Partial(Box::new(left))
                }))
            }
        }
    };

    (@bin-define $name:ident ($head:ident $( $tail:ident )+) $( $fun:tt )*) => {
        precedence!(@bin-define $name ($( $tail )+) $( $fun )*);
    };

    (@un ($name:ident { $( $fun:ident )* })) => {
        define!($name = UnOp <PrimitiveExpr <'code>>, PrimitiveExpr <'code>);

        impl <'code> Parse <'code> for $name <'code> {
            fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
                Result(Ok(match precedence!(@sign-fun input, $( $fun )*) {
                    Some(operator) => Self::Full(Box::new(UnOp {
                        value: PrimitiveExpr::parse(input)?,
                        operator
                    })),
                    None => Self::Partial(Box::new(PrimitiveExpr::parse(input)?))
                }))
            }
        }
    };

    // (@un ($name:ident [$( $fun:ident )*]) $( $tail:tt )+) => {
    // TODO
    // };

    (@sign-fun $input:ident, $single:ident) => {
        unspan($input.$single())
    };

    (@sign-fun $input:ident, $head:ident $( $tail:ident )+) => {
        precedence!(@sign-fun $input, $head).or_else(|| precedence!(@sign-fun $input, $( $tail )+))
    };

    (@rev($( $label:tt )*) [] | [$( $rev:tt )*]) => {
        precedence! { $( $label )* $( $rev )* }
    };

    (@rev($( $label:tt )*) [( $( $head:tt )* ) $( ( $( $tail:tt )* ) )*] | [$( $rev:tt )*]) => {
        precedence! { @rev($( $label )*) [$( ( $( $tail )* ) )*] | [( $( $head )* ) $( $rev )*] }
    };

    (@expr $last:ident) => {
        #[derive(Clone, Debug)]
        pub struct Expr <'code> {
            pub value: $last <'code>
        }

        impl <'code> Parse <'code> for Expr <'code> {
            fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
                Result(Ok(Self {
                    value: $last::parse(input)?
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
        PlusMinusExpr [plus minus]

    'binop:
        MulDivExpr  [star slash]
        SumDiffExpr [plus minus]
}

#[derive(Clone, Debug)]
pub enum PrimitiveExpr <'code> {
    Ident(Spanned <&'code str>)
}

impl <'code> Parse <'code> for PrimitiveExpr <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        Result(Ok(Self::Ident(input.ident_as_spanned_str()?)))
    }
}
