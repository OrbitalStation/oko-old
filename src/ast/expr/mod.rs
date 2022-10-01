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

fn unspan(token: &Token) -> Span {
    token.span
}

#[derive(Clone, Debug)]
pub struct Expr <'code> {
    pub value: SumDiffExpr <'code>
}

impl <'code> Parse <'code> for Expr <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        Result(Ok(Self {
            value: SumDiffExpr::parse(input)?
        }))
    }
}

#[derive(Clone, Debug)]
pub enum SumDiffExpr <'code> {
    Full(Box <BinOp <MulDivExpr <'code>>>),
    Partial(Box <MulDivExpr <'code>>)
}

impl <'code> Parse <'code> for SumDiffExpr <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let left = MulDivExpr::parse(input)?;

        Result(Ok(match input.plus().0.ok().map(unspan).or_else(|| input.minus().0.ok().map(unspan)) {
            Some(operator) => {
                let right = MulDivExpr::parse(input)?;

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

#[derive(Clone, Debug)]
pub enum MulDivExpr <'code> {
    Full(Box <BinOp <PlusMinusExpr <'code>>>),
    Partial(Box <PlusMinusExpr <'code>>)
}

impl <'code> Parse <'code> for MulDivExpr <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let left = PlusMinusExpr::parse(input)?;

        Result(Ok(match input.star().0.ok().map(unspan).or_else(|| input.slash().0.ok().map(unspan)) {
            Some(operator) => {
                let right = PlusMinusExpr::parse(input)?;

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

#[derive(Clone, Debug)]
pub enum PlusMinusExpr <'code> {
    Full(Box <UnOp <PrimitiveExpr <'code>>>),
    Partial(Box <PrimitiveExpr <'code>>)
}

impl <'code> Parse <'code> for PlusMinusExpr <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        Result(Ok(match input.plus().0.ok().map(unspan).or_else(|| input.minus().0.ok().map(unspan)) {
            Some(operator) => Self::Full(Box::new(UnOp {
                value: PrimitiveExpr::parse(input)?,
                operator
            })),
            None => Self::Partial(Box::new(PrimitiveExpr::parse(input)?))
        }))
    }
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
