use crate::*;
use core::fmt::{Debug, Formatter, Result as FmtResult};

#[inline]
fn parse_one <'code, const NEWLINES: u32, const EXTRA_LEN: u32> (
    pos: &mut CursorPosition,
    len: u32,
    kind: TokenKind <'code>,
    code: &mut &'code str
) -> Token <'code> {
    let oldpos = *pos;
    if NEWLINES > 0 {
        pos.line += NEWLINES;
        pos.column = 1
    } else {
        pos.column += len + EXTRA_LEN;
    }
    let span = Span {
        start: oldpos,
        end: *pos
    };
    *code = &(*code)[len as usize..];
    return Token {
        kind,
        span
    }
}

macro_rules! token {
    ($($( #[$( $attrs:tt )*] )* $name:ident $(($value:ty))? [$fn_name:ident] [$debugname:literal $($tt:tt)* ] )*) => {
        ///
        /// The kind of token
        ///
        #[derive(Clone, Eq, PartialEq)]
        pub enum TokenKind <'code> {$(
            $( #[$( $attrs )*] )*
            $name $(($value))?
        ),*}

        impl <'code> Debug for TokenKind <'code> {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                token!(@debug f, $( [self $name $($value,)? $debugname $($tt)*] )*)
            }
        }

        impl <'code> Token <'code> {
            /// Tries to parse a token from the text
            /// with the beginning at some position
            ///
            /// `code` should never be an empty string
            ///
            /// Mutates the `pos` onto new start location on success
            /// Mutates the `code` to exclude the parsed symbols on success
            ///
            /// Returns the parsed token
            pub fn parse(pos: &mut CursorPosition, code_remaining: &mut &'code str, filename: &str, full_code: &str) -> Result <Self> {
                debug_assert!(!code_remaining.is_empty());

                $(
                    token!(@parse code_remaining, pos, $name $(($value))? $debugname $($tt)*);
                )*

                let le_problem_place = &code_remaining[..1.max(code_remaining.find(char::is_whitespace).unwrap_or(code_remaining.len()))];

                Result(Err(Error {
                    span: Span::extend_by_one(*pos),
                    message: String::from(concat!("expected one of ", token!(@message $( [$name $(($value))? $debugname $($tt)* ] )*))),
                    clarifying: format!("...but got `{le_problem_place}`"),
                    filename: filename.to_string(),
                    code: full_code.to_string()
                }))
            }

            pub fn to_spanned_str(&self) -> Option <Spanned <&'code str,>> {
                Some(token!(@spanned self $([$name $($value)?])*))
            }
        }

        impl <'code> ParseInput <'code> {$(
            pub fn $fn_name <'s> (&'s mut self) -> Result <&'s Token <'code>> {
                let next = self.peek($debugname)?;

                if matches!(&self.stream.buf[next].kind, token!(@matches $name $($value)?)) {
                    self.go_forward();
                    return Result(Ok(&self.stream.buf[next]))
                }

                self.generate_expected_err(concat!('`', $debugname, '`'), &self.stream.buf[next])
            }
        )*}
    };

    (@spanned $self:ident [$name:ident $value:ty] $( $tail:tt )*) => {
        if let TokenKind::$name(value) = $self.kind {
            Spanned {
                data: value,
                span: $self.span
            }
        } else {
            token!(@spanned $self $( $tail )*)
        }
    };

    (@spanned $self:ident [$name:ident] $( $tail:tt )*) => {
        token!(@spanned $self $( $tail )*)
    };

    (@spanned $self:ident) => {
        return None
    };

    (@matches $name:ident) => {
        TokenKind::$name
    };

    (@matches $name:ident $value:ty) => {
        TokenKind::$name(_)
    };

    (@debug $f:ident, [$self:ident $name:ident $value:ty, $debugname:literal $( $tt:tt )*] $( $tail:tt )*) => {
        if let Self::$name(val) = $self {
            $f.write_fmt(format_args!("\"{}\"", val))
        } else {
            token!(@debug $f, $( $tail )*);
        }
    };

    (@debug $f:ident, [$self:ident $name:ident $pat:literal $( $tt:tt )*] $( $tail:tt )*) => {
        if let Self::$name = $self {
            $f.write_fmt(format_args!("\"{}\"", $pat.escape_debug()))
        } else {
            token!(@debug $f, $( $tail )*)
        }
    };

    (@debug $f:ident,) => {
        unreachable!()
    };

    (@message [$($head:tt)*] [$($tail:tt)*]) => {
        concat!(token!(@messageSingle $($head)*), " or ", token!(@messageSingle $($tail)*), "...")
    };

    (@message [$($head:tt)*] $( [ $($tail:tt)* ] )+) => {
        concat!(token!(@messageSingle $($head)*), ", ", token!(@message $( [ $($tail)* ] )*))
    };

    (@messageSingle $name:ident($vl:ty) $debugname:literal $($tt:tt)*) => {
        $debugname
    };

    (@messageSingle $name:ident $pat:literal, newlines = 1 $( $tt:tt )*) => {
        "a newline character"
    };

    (@messageSingle Tab $pat:literal $( $tt:tt )*) => {
        "a tab character"
    };

    (@messageSingle $name:ident $pat:literal $( $tt:tt )*) => {
        concat!('`', $pat, '`')
    };

    (@parse $textcode:ident, $pos:ident, $name:ident($vl:ty) $debugname:literal $( $code:tt )*) => {
        if let Some(len) = ($( $code )*)($textcode) {
            return Result(Ok(parse_one::<0, 0>($pos, len, TokenKind::$name(&$textcode[..len as usize]), $textcode)))
        }
    };

    (@parse $code:ident, $pos:ident, $name:ident $pat:literal, tabs = $tabs:literal) => {
        if $code.starts_with($pat) {
            return Result(Ok(parse_one::<0, { $tabs * SPACES_IN_TAB - $pat.len() as u32 }>($pos, $pat.len() as u32, TokenKind::$name, $code)))
        }
    };

    (@parse $code:ident, $pos:ident, $name:ident $pat:literal $( , newlines = $newlines:literal )?) => {
        if $code.starts_with($pat) {
            const EIZ: u32 = token!(@emptyIsZero $( $newlines )?);
            return Result(Ok(parse_one::<EIZ, 0>($pos, $pat.len() as u32, TokenKind::$name, $code)))
        }
    };

    (@emptyIsZero) => {
        0
    };

    (@emptyIsZero $literal:literal) => {
        $literal
    };
}

token! {
    /// The tab character(emitted only after Newline or another Tab,
    ///     sequence of spaces otherwise).
    ///
    /// Is either a `\t` sign or a space repeated 4 times, i.e. `    `
    Tab[tab]["\t", tabs = 1]

    /// ->
    Arrow[arrow]["->"]

    /// :
    TwoDots[two_dots][":"]

    /// +
    Plus[plus]["+"]

    /// -
    Minus[minus]["-"]

    /// *
    Star[star]["*"]

    /// /
    Slash[slash]["/"]

    /// ,
    Comma[comma][","]

    /// =
    Eq[eq]["="]

    /// The newline character
    Newline[newline]["\n", newlines = 1]

    /// The string that starts with an alphabetic character and continues with alphanumeric
    ///
    /// Examples: `H`, `twentySeven`, `mom`, `abc1234`, `ja83n82bjd9q`
    Ident(&'code str)[ident]["an identifier" |code: &str| {
        let first = code.chars().next().unwrap();
        if first.is_alphabetic() {
            Some(code[first.len_utf8()..].find(|char: char| !char.is_alphanumeric()).unwrap_or(code.len()) as u32 + 1)
        } else {
            None
        }
    }]
}

///
/// The token -- smallest logical piece of the text
///
#[derive(Clone)]
pub struct Token <'code> {
    pub kind: TokenKind <'code>,
    pub span: Span
}

impl <'code> Token <'code> {
    pub const STUB: Token <'code> = Token {
        kind: TokenKind::Newline,
        span: Span::DEFAULT
    };
}

impl <'code> Debug for Token <'code> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
        self.kind.fmt(f)
    }
}
