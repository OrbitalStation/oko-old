#![feature(try_trait_v2)]
#![feature(adt_const_params)]

#![allow(incomplete_features)]

mod token;
pub use token::{TokenKind, Token, TokenStream};

mod ast;
pub use ast::{
    RawTypeBase, TypeIndex, Parse, ParseInput, ParseDebug,
    TypeBaseContainer, TypedVariablesSet, Fn, Punctuated, print_punctuated_seq,
    Expr, RawFnBodyBase, FnBodyIndex, FnBodyContainer, RawTypeDefinition,
    Item, ParseFun, BakedTypeBase, BakedTypeBaseKind, BUILTIN_BAKED_TYPES
};

mod span;
pub use span::{CursorPosition, Span, Spanned};

mod error;
pub use error::{Result, Error};

pub const SPACES_IN_TAB: u32 = 4;

#[macro_export]
macro_rules! const_array {
    ($vis:vis const $name:ident : [$ty:ty] = [$( $expr:expr ),*]) => {
        $vis const $name: [$ty; const_array!(@ $( $expr ),*)] = [$( $expr )*];
    };

    (@ $expr:expr) => {
        1
    };

    (@ $head:expr, $( $tail:expr ),+) => {
        const_array!(@ $head) + const_array!(@ $( $tail )*)
    };
}
