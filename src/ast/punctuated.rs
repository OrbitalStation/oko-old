use crate::*;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::marker::PhantomData;

pub struct Punctuated <'code, T, const S: &'static str> {
    pub vec: Vec <T>,
    _marker: PhantomData <&'code ()>
}

impl <'code, T: Clone, const S: &'static str> Clone for Punctuated <'code, T, S> {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
            _marker: PhantomData
        }
    }
}

impl <'code, T: ParseDebug, const S: &'static str> ParseDebug for Punctuated <'code, T, S> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        print_punctuated_seq::<_, S>(self.vec.iter().map(|i| i.debug(input)), f)
    }
}

pub fn print_punctuated_seq <T: Debug, const S: &'static str> (mut iter: impl DoubleEndedIterator <Item = T>, f: &mut Formatter <'_>) -> FmtResult {
    let last = match iter.next_back() {
        Some(ok) => ok,
        None => return Ok(())
    };
    for item in iter {
        item.fmt(f)?;
        f.write_str(S)?
    }
    last.fmt(f)
}

pub type ParseFun <'code> = for <'a> fn(&'a mut ParseInput <'code>) -> Result <&'a Token <'code>>;

impl <'code, T, const S: &'static str> Punctuated <'code, T, S> {
    pub fn single(value: T) -> Self {
        Self {
            vec: vec![value],
            _marker: PhantomData
        }
    }

    pub const fn wrap(vec: Vec <T>) -> Self {
        Self {
            vec,
            _marker: PhantomData
        }
    }

    pub fn new_with_custom_parser(
        input: &mut ParseInput <'code>,
        parser: impl for <'a> core::ops::Fn(&'a mut ParseInput <'code>) -> Result <T>,
        sep: ParseFun <'code>,
        stop: ParseFun <'code>
    ) -> Result <Self> {
        let mut vec = vec![];

        loop {
            if let Result(Ok(_)) = stop(input) {
                break
            }

            let parsed = parser(input)?;

            vec.push(parsed);

            if let Result(Ok(_)) = stop(input) {
                break
            }

            sep(input)?;
        }

        Result(Ok(Self {
            vec,
            _marker: PhantomData
        }))
    }
}

impl <'code, T: Parse <'code>, const S: &'static str> Punctuated <'code, T, S> {
    ///
    /// Parses `T` from `input`, each `T` is separated by `sep`.
    ///
    /// Stops when `stop` is parsed.
    ///
    /// Fails if either `T` or `sep` failed to parse
    ///
    pub fn new(
        input: &mut ParseInput <'code>,
        sep: ParseFun <'code>,
        stop: ParseFun <'code>
    ) -> Result <Self> {
        Self::new_with_custom_parser(input, T::parse, sep, stop)
    }
}
