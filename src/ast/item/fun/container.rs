use crate::*;
use core::fmt::{Formatter, Result as FmtResult};

#[derive(Clone)]
#[repr(u8)]
pub enum FnBodyContainer <'code> {
    Raw(Vec <RawFnBodyBase <'code>>),
    Baked(Vec <BakedFnBodyBase <'code>>)
}

impl <'code> ParseDebug for FnBodyContainer <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        let mut list = f.debug_list();

        match self {
            Self::Raw(raw) => list.entries(raw),
            Self::Baked(baked) => list.entries(baked.iter().map(|x| x.debug(input))),
        };

        list.finish()
    }
}

impl <'code> FnBodyContainer <'code> {
    pub const fn new() -> Self {
        Self::Raw(vec![])
    }
}
