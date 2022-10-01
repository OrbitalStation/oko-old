use core::fmt::{Debug, Result, Formatter};

///
/// The position of an item in file
///
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CursorPosition {
    ///
    /// Starts from 1
    ///
    pub line: u32,

    ///
    /// Starts from 1
    ///
    pub column: u32
}

impl CursorPosition {
    pub const DEFAULT: Self = CursorPosition {
        line: 1,
        column: 1
    };
}

impl Debug for CursorPosition {
    fn fmt(&self, f: &mut Formatter <'_>) -> Result {
        f.write_fmt(format_args!("{}:{}", self.line, self.column))
    }
}

/// The precise position of an item in the file
#[derive(Copy, Clone)]
pub struct Span {
    ///
    /// The pointer to first symbol of an item
    ///
    pub start: CursorPosition,

    ///
    /// The pointer to the symbol *after* the last symbol of an item
    ///
    pub end: CursorPosition
}

impl Span {
    pub const DEFAULT: Span = Span {
        start: CursorPosition::DEFAULT,
        end: CursorPosition::DEFAULT
    };
}

impl Span {
    /// Extends the cursor position to span by making the end out of it
    pub const fn extend_by_one(pos: CursorPosition) -> Self {
        Self {
            start: pos,
            end: CursorPosition {
                line: pos.line,
                column: pos.column + 1
            }
        }
    }

    /// Returns the lines in `content` that is spanned by `self`
    pub fn get_spanned_lines(self, code: &str) -> Vec <&str> {
        let mut lines = code
            .split('\n')
            .skip((self.start.line - 1) as usize)
            .take((self.end.line - self.start.line + 1) as usize)
            .collect::<Vec <_>>();

        lines[0] = &lines[0][self.start.column as usize - 1..];

        let last = lines.last_mut().expect("no lines in span");
        *last = &last[..(self.end.column as usize - 1).min(last.len())];
        if let Some(idx) = last.find(char::is_whitespace) {
            *last = &last[..idx]
        }

        lines
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter <'_>) -> Result {
        f.write_fmt(format_args!("Span({:?}..{:?})", self.start, self.end))
    }
}

#[derive(Clone)]
pub struct Spanned <T> {
    pub data: T,
    pub span: Span
}

impl <T: Copy> Copy for Spanned <T> {}

impl <T: Debug> Debug for Spanned <T> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter <'_>) -> Result {
        self.data.fmt(f)
    }
}

impl <T: PartialEq> PartialEq for Spanned <T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }

    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        self.data.ne(&other.data)
    }
}

impl <T: Eq> Eq for Spanned <T> {}
