use crate::*;
use core::fmt::{Debug, Formatter, Write, Result as FmtResult};

///
/// Represents a raw token sequence
///
pub struct TokenStream <'code> {
    /// The container of all the tokens for a file
    pub buf: Vec <Token <'code>>,

    /// The pointer to the current element in the `buf`
    pub cur: usize
}

impl <'code> Debug for TokenStream <'code> {
    fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
        for token in &self.buf {
            f.write_char('[')?;
            token.fmt(f)?;
            f.write_char(']')?;
            f.write_char(if token.kind == TokenKind::Newline {
                '\n'
            } else {
                ' '
            })?
        }

        f.write_char('\n')?;
        f.write_fmt(format_args!("cur: {}", self.cur))?;
        if self.cur < self.buf.len() {
            f.write_fmt(format_args!("= {:?}", self.buf[self.cur]))?;
        }

        Ok(())
    }
}

impl <'code> TokenStream <'code> {
    pub const fn empty() -> Self {
        Self::from(vec![])
    }

    pub const fn from(buf: Vec <Token <'code>>) -> Self {
        Self {
            buf,
            cur: 0
        }
    }

    ///
    /// Creates new [`TokenStream`] from the source code `code` of file named `filename`
    ///
    pub fn new(filename: &str, code: &'code str) -> Result <Self> {
        let mut buf = vec![];
        let mut cursor_position = CursorPosition::DEFAULT;
        let mut remaining_code = code;

        remove_spaces(&mut remaining_code, &mut cursor_position);
        while !remaining_code.is_empty() {
            let token = Token::parse(&mut cursor_position, &mut remaining_code, filename, code)?;

            // Prevent tabs from being emitted if they are not following a newline or an another tab,
            // i.e. tabs are interpreted as tabs only if they are in the beginning of the line
            // and there are no other symbols than tabs between this tab and the beginning of the line,
            // treated as space otherwise
            if !(token.kind == TokenKind::Tab && buf.last().map(|x: &Token| x.kind != TokenKind::Newline && x.kind != TokenKind::Tab).unwrap_or(false)) {
                buf.push(token)
            }

            remove_spaces(&mut remaining_code, &mut cursor_position);
        }

        Result(Ok(Self {
            buf,
            cur: 0
        }))
    }
}

fn remove_spaces(code: &mut &str, cursor_pos: &mut CursorPosition) {
    let index = code.find(|char: char| char != ' ').unwrap_or(0);
    *code = &code[index..];
    cursor_pos.column += index as u32;
}
