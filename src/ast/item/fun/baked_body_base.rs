use crate::*;
use core::fmt::{Formatter, Result as FmtResult};

#[derive(Clone)]
pub enum BakedFnBodyStmt <'code> {
    Expr(Expr <'code>),
}

#[derive(Clone)]
pub struct BakedFnBodyBase <'code> {
    pub body: Vec <Expr <'code>>
}

impl <'code> ParseDebug for BakedFnBodyBase <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        print_punctuated_seq::<_, "\n">(self.body.iter().map(|i| i.debug(input)), f)
    }
}
