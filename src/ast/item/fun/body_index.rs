use crate::*;
use core::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
pub struct FnBodyIndex {
    pub base_index: u32
}

impl <'code> Parse <'code> for FnBodyIndex {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let base = RawFnBodyBase::parse(input)?;

        let base_index = input.add_raw_fn_body_base(base);

        Result(Ok(Self {
            base_index
        }))
    }
}

impl ParseDebug for FnBodyIndex {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        match &input.fn_body_bases {
            FnBodyContainer::Raw(raw) => raw[self.base_index as usize].fmt(f),
            FnBodyContainer::Baked(baked) => baked[self.base_index as usize].debug_impl(input, f)
        }
    }
}
