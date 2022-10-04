use crate::*;
use core::fmt::{Debug, Formatter, Write, Result as FmtResult};

#[derive(Clone)]
pub struct Fn <'code> {
    pub name: Spanned <&'code str>,
    pub args: Vec <TypedVariable <'code>>,
    pub ret_ty: TypeIndex,
    pub body: FnBodyIndex
}

impl <'code> ParseDebug for Fn <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
		self.name.fmt(f)?;
        f.write_char(' ')?;
		print_punctuated_seq::<_, ", ">(self.args.iter().map(|i| i.debug(input)), f)?;

        if !self.ret_ty.is_unit_tuple() {
            f.write_str(" -> ")?;
            self.ret_ty.debug_impl(input, f)?;
        }

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
        fn stop(input: &mut ParseInput) -> Option <bool> {
            if input.arrow().0.is_ok() {
                Some(true)
            } else if input.is_exhausted() || unsafe { check_if_the_next_token_is_newline(input) } {
                Some(false)
            } else {
                None
            }
        }

        let name = input.ident_as_spanned_str()?;

        let mut args = vec![];

        let stopped_by_arrow = loop {
            if let Some(x) = stop(input) {
                break x
            }

            let parsed = TypedVariable::parse(input)?;

            args.extend(parsed);

            if let Some(x) = stop(input) {
                break x
            }

            input.comma()?;
        };

        let ret_ty = if stopped_by_arrow {
            TypeIndex::parse(input)?
        } else {
            TypeIndex::UNIT_TUPLE
        };

        let body = FnBodyIndex::parse(input)?;

        Result(Ok(Self {
            name,
            args,
            ret_ty,
            body
        }))
    }
}
