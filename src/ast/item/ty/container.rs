use crate::*;
use core::fmt::{Formatter, Result as FmtResult};

#[derive(Clone)]
#[repr(u8)]
pub enum TypeBaseContainer <'code> {
    /// `Raw` is a type just parsed from a file with no extra processing
    /// performed over it
    Raw(Vec <RawTypeBase <'code>>),

    /// `Baked` is a type that went through all the processing and information collecting,
    /// ready to be used and transpiled into the backend
    Baked(Vec <BakedTypeBase <'code>>)
}

impl <'code> ParseDebug for TypeBaseContainer <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        let mut list = f.debug_list();

        match self {
            Self::Raw(raw) => list.entries(raw.iter().map(|x| x.debug(input))),
            Self::Baked(baked) => list.entries(baked.iter().map(|x| x.debug(input))),
        };

        list.finish()
    }
}

impl <'code> TypeBaseContainer <'code> {
    pub const fn new() -> Self {
        Self::Raw(vec![])
    }
}
