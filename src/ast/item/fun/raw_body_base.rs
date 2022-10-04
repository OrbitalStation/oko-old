use crate::*;
use core::ptr::NonNull;
use core::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
pub struct RawFnBodyBase <'code> {
    body: NonNull <[Token <'code>]>
}

impl <'code> Parse <'code> for RawFnBodyBase <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        input.newline()?;

		let body = input.find_end_of_block_and_return_everything_in_it_and_also_go_forward_to_its_end(1);

        Result(Ok(Self {
            body
        }))
    }
}

impl <'code> RawFnBodyBase <'code> {
    /// # Safety
    ///
    /// At the moment of call of this function the `input`
    /// that was passed to it should still be valid
    pub fn body(&self) -> &'code [Token <'code>] {
        // SAFETY: caller must uphold the contract
        unsafe { self.body.as_ref() }
    }
}

impl <'code> Debug for RawFnBodyBase <'code> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
        self.body().fmt(f)
    }
}

