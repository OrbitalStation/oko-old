use crate::*;

pub fn remove_newlines(input: &mut ParseInput) {
    general_remove(input, |kind| matches!(kind, TokenKind::Newline))
}

pub fn remove_newlines_and_tabs(input: &mut ParseInput) {
    general_remove(input, |kind| matches!(kind, TokenKind::Newline | TokenKind::Tab))
}

pub unsafe fn check_if_the_next_token_is_newline(input: &mut ParseInput) -> bool {
    check_if_the_next_token_is(input, |kind| matches!(kind, TokenKind::Newline))
}

fn general_remove(input: &mut ParseInput, cond: impl for <'a, 'code> core::ops::Fn(&'a TokenKind <'code>) -> bool) {
    loop {
        if input.is_exhausted() {
            break
        }

        if unsafe { check_if_the_next_token_is(input, &cond) } {
            input.go_forward()
        } else {
            break
        }
    }
}

unsafe fn check_if_the_next_token_is(input: &mut ParseInput, cond: impl for <'a, 'code> core::ops::Fn(&'a TokenKind <'code>) -> bool) -> bool {
    let next = input.stream.buf.get_unchecked(input.peek("").0.ok().unwrap_unchecked());
    cond(&next.kind)
}
