use crate::*;
use std::process::{ExitCode, Termination};
use core::fmt::Display;
use core::ops::{Try, FromResidual, ControlFlow};
use core::convert::Infallible;
use owo_colors::*;

#[repr(transparent)]
pub struct Result <T, E = Error> (pub core::result::Result <T, E>);

impl <T> Result <T> {
    pub fn with_custom_err_message(mut self, msg: impl core::ops::Fn() -> String) -> Self {
        if let Err(ref mut err) = self.0 {
            err.message = msg()
        }

        self
    }
}

impl <T> FromResidual for Result <T> {
    fn from_residual(residual: Result <Infallible>) -> Self {
        Self(Err(match residual.0 {
            Err(err) => err,
            Ok(_) => unsafe { core::hint::unreachable_unchecked() }
        }))
    }
}

impl <T> Try for Result <T> {
    type Output = T;

    type Residual = Result <Infallible>;

    #[inline(always)]
    fn from_output(output: Self::Output) -> Self {
        Self(Ok(output))
    }

    #[inline]
    fn branch(self) -> ControlFlow <Self::Residual, Self::Output> {
        match self.0 {
            Ok(ok) => ControlFlow::Continue(ok),
            Err(err) => ControlFlow::Break(Result::<Infallible>(Err(err)))
        }
    }
}

impl Termination for Result <()> {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(()) => ().report(),
            Err(err) => err.report()
        }
    }
}

///
/// The resulting error explaining in details why something went wrong
///
#[derive(Debug)]
pub struct Error {
    /// The span of the problem in the file
    pub span: Span,

    /// The general message, printed on the top
    pub message: String,

    /// The clarification message, printed near the problem place
    pub clarifying: String,

    /// The name of the file where error occurred
    pub filename: String,

     /// The code of the file where error occurred
    pub code: String
}

impl Error {
    pub const STUB: Error = Error {
        span: Span::DEFAULT,
        message: String::new(),
        clarifying: String::new(),
        filename: String::new(),
        code: String::new()
    };
}

impl Termination for Error {
    fn report(self) -> ExitCode {
        let ladjust = " ".repeat(self.span.start.line.max(self.span.end.line).to_string().len() + 1);

        print!("{}{} ", "error".bright_red().bold(), ":".bold());
        print_with_style_and_green_if_asterisks(&self.message, |v| print!("{}", v.bold()));
        println!("{}", ":".bold());

        println!("{}{} {}:{:?}", &ladjust[1..], "-->".blue().bold(), self.filename, self.span.start);
        println!("{ladjust}{}", "|".blue().bold());

        for (linenum, line) in self.span.get_spanned_lines(&self.code).iter().enumerate() {
            let linenum = linenum as u32;

            let idx = linenum + self.span.start.line;
            let idx_stringified = idx.to_string();
            let full_line = self.code.split('\n').nth(idx as usize - 1).unwrap();

            let ladjust2 = " ".repeat(ladjust.len() - idx_stringified.len() - 1);

            let circumflex_ladjsust = if linenum == 0 {
                " ".repeat(self.span.start.column as usize)
            } else {
                String::new()
            } + &" ".repeat(full_line.matches('\t').count() * (SPACES_IN_TAB - 1) as usize);

            // let underscoring_len = if line == "<EOF>" {
            //     1
            // } else {
            //     line.len()
            // };

            print!("{ladjust2}{idx} {stick} {line}\n{ladjust}{stick}{circumflex_ladjsust}{underscoring} ",
                idx = idx_stringified.blue().bold(),
                stick = "|".blue().bold(),
                line = full_line.red(),
                underscoring = "^".repeat(1.max(line.len())).bright_red().bold())
        }

        print_with_style_and_green_if_asterisks(&self.clarifying, |v| print!("{}", v.bright_red().bold()));
        println!();

        ExitCode::FAILURE
    }
}

type Printer = fn(&dyn Display);

fn print_with_style_and_green_if_asterisks(message: &str, default: Printer) {
    if let Some(start) = message.find('`') {
        let extra = start + '`'.len_utf8();

        let end = message[extra..].find('`').unwrap_or(message.len() - extra) + extra;

        default(&&message[..start]);
        print!("{}", (&message[start..=end]).green().bold());
        print_with_style_and_green_if_asterisks(&message[end + message[end..].chars().next().map(char::len_utf8).unwrap_or(0)..], default)
    } else {
        default(&message)
    }
}
