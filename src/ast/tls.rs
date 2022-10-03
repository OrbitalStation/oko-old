use crate::*;

pub fn remove_newlines(input: &mut ParseInput) {
    general_remove(input, |kind| matches!(kind, TokenKind::Newline))
}

pub fn remove_newlines_and_tabs(input: &mut ParseInput) {
    general_remove(input, |kind| matches!(kind, TokenKind::Newline | TokenKind::Tab))
}

pub fn general_remove(input: &mut ParseInput, cond: impl for <'a, 'code> core::ops::Fn(&'a TokenKind <'code>) -> bool) {
    loop {
        if input.is_exhausted() {
            break
        }

        // SAFETY: out-of-bounds case checked above
        let next = unsafe { input.stream.buf.get_unchecked(input.peek("").0.ok().unwrap_unchecked()) };

        if cond(&next.kind) {
            input.go_forward()
        } else {
            break
        }
    }
}
