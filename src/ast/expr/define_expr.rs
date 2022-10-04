#[macro_export]
macro_rules! define_expr {
    ($name:ident = $full:ty, $partial:ty) => {
        #[derive(Clone)]
        pub enum $name <'code> {
            Full(Box <$full>),
            Partial(Box <$partial>)
        }

        impl <'code> GetSpan for $name <'code> {
            fn span(&self) -> Span {
                match self {
                    Self::Full(full) => full.span(),
                    Self::Partial(partial) => partial.span()
                }
            }
        }

        impl <'code> ParseDebug for $name <'code> {
            fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
                f.write_str(concat!(stringify!($name), "::"))?;
                match self {
                    Self::Full(full) => f.debug_tuple("Full")
                        .field(&full.debug(input))
                        .finish(),
                    Self::Partial(partial) => f.debug_tuple("Partial")
                        .field(&partial.debug(input))
                        .finish(),
                }
            }
        }
    };
}
