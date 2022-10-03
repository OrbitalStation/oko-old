use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::ptr::NonNull;
use crate::*;

pub trait ParseDebug: Sized {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult;

    #[inline(always)]
    fn debug <'input, 'code> (&'code self, input: &'input ParseInput <'code>) -> ParseDebugHelper <'input, 'code, Self> {
        ParseDebugHelper {
            data: self,
            input
        }
    }
}

pub struct ParseDebugHelper <'input, 'code, T: ParseDebug> {
    data: &'code T,
    input: &'input ParseInput <'code>
}

impl <'input, 'code, T: ParseDebug> Debug for ParseDebugHelper <'input, 'code, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
        self.data.debug_impl(self.input, f)
    }
}

pub struct ParseInput <'code> {
    pub stream: TokenStream <'code>,
    pub code: &'code str,
    pub filename: &'code str,
    pub type_bases: TypeBaseContainer <'code>,
    pub fn_body_bases: FnBodyContainer <'code>,
    pub top_level_items: Vec <Item <'code>>
}

impl <'code> Debug for ParseInput <'code> {
    fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
        struct Inner <'code, 'a> {
            input: &'a ParseInput <'code>
        }

        impl <'code, 'a> Debug for Inner <'code, 'a> {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                f.debug_list()
                    .entries(self.input.top_level_items.iter().map(|x| x.debug(&self.input)))
                    .finish()
            }
        }

        f.debug_struct("ParseInput")
            .field("stream", &self.stream)
            .field("code", &self.code)
            .field("filename", &self.filename)
            .field("type_bases", &self.type_bases.debug(self))
            .field("fn_body_bases", &self.fn_body_bases.debug(self))
            .field("top_level_items", &Inner { input: self })
            .finish()
    }
}

impl <'code> ParseInput <'code> {
    pub fn ident_as_spanned_str(&mut self) -> Result <Spanned <&'code str>> {
        // SAFETY: ident returns an identifier which has its str
        Result(Ok(unsafe { self.ident()?.to_spanned_str().unwrap_unchecked() }))
    }

    pub fn keyword(&mut self, keyword: &str) -> Result <()> {
        let next = self.ident().with_custom_err_message(|| format!("expected a keyword `{keyword}`..."))?;

        if let Token { kind: TokenKind::Ident(ident), .. } = next {
            if *ident == keyword {
                return Result(Ok(()))
            }
        } else {
            unreachable!()
        }

        let next = next as *const Token <'code>;
        let next = unsafe { core::mem::transmute(&core::ptr::read(next)) };
        self.generate_expected_err(&format!("a keyword `{keyword}`"), next)
    }

    pub fn generate_expected_err <T> (&self, message: &str, next: &Token) -> Result <T> {
        Result(Err(Error {
            span: next.span,
            message: format!("expected {message}..."),
            clarifying: format!("...but got {next:?}"),
            filename: self.filename.to_string(),
            code: self.code.to_string()
        }))
    }

    pub const fn is_type_base_container_raw(&self) -> bool {
        matches!(&self.type_bases, TypeBaseContainer::Raw(_))
    }

    pub fn find_or_add_raw_type_base(&mut self, name: Spanned <&'code str>) -> u32 {
        (match &mut self.type_bases {
            TypeBaseContainer::Raw(raw) => if let Some(idx) = raw
                .iter()
                .enumerate()
                .find(|(_, x)| x.name() == name)
                .map(|(x, _)| x) {
                idx
            } else {
                let idx = raw.len();
                raw.push(RawTypeBase::Stub(name));
                idx
            },
            _ => unreachable!()
        }) as u32
    }

    pub fn add_raw_fn_body_base(&mut self, base: RawFnBodyBase <'code>) -> u32 {
        (match &mut self.fn_body_bases {
            FnBodyContainer::Raw(raw) => {
                let idx = raw.len();
                raw.push(base);
                idx
            },
            _ => unreachable!()
        }) as u32
    }

    ///
    /// Returns the current token without moving forward
    ///
    pub fn peek(&self, err_message: &str) -> Result <usize> {
        if self.is_exhausted() {
            return Result(Err(Error {
                span: Span::extend_by_one(self.stream.buf.last().map(|x| x.span.end).unwrap_or(CursorPosition::DEFAULT)),
                message: err_message.to_string(),
                clarifying: String::from("got <EOF>"),
                filename: self.filename.to_string(),
                code: self.code.to_string()
            }))
        }

        // SAFETY: out-of-bounds case checked above
        Result(Ok(self.stream.cur))
    }

    ///
    /// Finds the end of the current block and sets `self.cur` to the end of it
    ///
    /// `self.cur` should point to the very first token of the block
    ///
    /// `block_nesting_level` means the amount of tabs before each instruction in a block
    ///
    /// Returns content of the block
    ///
    pub fn find_end_of_block_and_return_everything_in_it_and_also_go_forward_to_its_end(&mut self, block_nesting_level: u8) -> NonNull <[Token <'code>]> {
        let mut tabs = 0;

        for (idx, token) in self.stream.buf[self.stream.cur..].iter().enumerate() {
            if token.kind == TokenKind::Tab {
                tabs += 1;
            } else if token.kind == TokenKind::Newline {
                // if start < idx {
                //     result.extend(self.stream.buf[self.stream.cur + start..self.stream.cur + idx].iter().map(|x| unsafe { NonNull::new_unchecked(x as *const Token as *mut _) }))
                // }

                tabs = 0;
            } else if tabs != block_nesting_level {
                let result = unsafe { NonNull::new_unchecked(&self.stream.buf[self.stream.cur..self.stream.cur + idx] as *const [Token] as *mut _) };
                self.stream.cur += idx;
                return result
            }
        }

        let result = unsafe { NonNull::new_unchecked(&self.stream.buf[self.stream.cur..] as *const [Token] as *mut _) };
        self.stream.cur = self.stream.buf.len() - 1;
        result
    }

    #[inline(always)]
    pub fn go_forward(&mut self) {
        self.stream.cur += 1
    }

    #[inline(always)]
    pub fn get(&self) -> usize {
        self.stream.cur
    }

    #[inline(always)]
    pub fn set(&mut self, cur: usize) {
        self.stream.cur = cur
    }

    pub fn is_exhausted(&self) -> bool {
        self.stream.cur >= self.stream.buf.len()
    }
}

pub trait Parse <'code>: Sized {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self>;

    #[inline(always)]
    fn parse(input: &mut ParseInput <'code>) -> Result <Self> {
        Result(Self::parse_with_returning_cur(input).0.map_err(|(_, e)| e))
    }

    fn parse_with_returning_cur(input: &mut ParseInput <'code>) -> Result <Self, (usize, Error)> {
        let old = input.get();
        let parsed = Self::parse_impl(input).0;
        Result(match parsed {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let new = input.get();
                input.set(old);
                Err((new - old, err))
            }
        })
    }
}
