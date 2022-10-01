use crate::*;
use core::fmt::{Debug, Formatter, Write, Result as FmtResult};
use core::ptr::NonNull;

#[derive(Clone)]
pub struct RawFnBodyBase <'code> {
    body: Vec <NonNull <[Token <'code>]>>
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
    pub fn body(&self) -> &Vec <&'code [Token <'code>]> {
        // SAFETY: caller must uphold the contract
        unsafe { core::mem::transmute(&self.body) }
    }
}

impl <'code> Debug for RawFnBodyBase <'code> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
        self.body().fmt(f)
    }
}

#[derive(Clone)]
pub struct FnBodyIndex {
    base_index: u32
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
            FnBodyContainer::Raw(raw) => raw[self.base_index as usize].fmt(f)
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum FnBodyContainer <'code> {
    Raw(Vec <RawFnBodyBase <'code>>)
}

impl <'code> FnBodyContainer <'code> {
    pub const fn new() -> Self {
        Self::Raw(vec![])
    }
}

#[derive(Clone)]
pub struct Fn <'code> {
    pub name: Spanned <&'code str>,
    pub args: Punctuated <'code, TypedVariablesSet <'code>, ", ">,
    pub ret_ty: TypeIndex,
    pub body: FnBodyIndex
}

impl <'code> ParseDebug for Fn <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
		self.name.fmt(f)?;
        f.write_char(' ')?;
		self.args.debug_impl(input, f)?;
        f.write_str(" -> ")?;
        self.ret_ty.debug_impl(input, f)?;
        f.write_char(' ')?;
        self.body.debug_impl(input, f)?;

        Ok(())
    }
}

impl <'code> Parse <'code> for Fn <'code> {
    ///
    /// Parses raw fn, i.e. with body left unparsed
    ///
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let name = input.ident_as_spanned_str()?;

        let args = Punctuated::new(input, ParseInput::comma, ParseInput::arrow)?;

        let ret_ty = TypeIndex::parse(input)?;

        let body = FnBodyIndex::parse(input)?;

        Result(Ok(Self {
            name,
            args,
            ret_ty,
            body
        }))
    }
}
