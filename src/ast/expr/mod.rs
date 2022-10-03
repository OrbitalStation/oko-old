use core::fmt::{Debug, Formatter, Result as FmtResult};
use crate::*;

pub trait GetSpan {
    fn span(&self) -> Span;
}

#[derive(Clone)]
pub struct UnOp <T> {
    pub value: T,
    pub operator: Span
}

impl <T: GetSpan> GetSpan for UnOp <T> {
    fn span(&self) -> Span {
        Span {
            start: self.operator.start,
            end: self.value.span().end
        }
    }
}

impl <T: ParseDebug> ParseDebug for UnOp <T> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("UnnOp")
            .field("left", &self.value.debug(input))
            .field("operator", &self.operator.get_spanned_lines(&input.code)[0])
            .finish()
    }
}

#[derive(Clone)]
pub struct BinOp <T> {
    pub left: T,
    pub right: T,
    pub operator: Span
}

impl <T: GetSpan> GetSpan for BinOp <T> {
    fn span(&self) -> Span {
        Span {
            start: self.left.span().start,
            end: self.right.span().end
        }
    }
}

impl <T: ParseDebug> ParseDebug for BinOp <T> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("BinOp")
            .field("left", &self.left.debug(input))
            .field("right", &self.right.debug(input))
            .field("operator", &self.operator.get_spanned_lines(&input.code)[0])
            .finish()
    }
}

#[inline]
fn unspan(token: Result <&Token>) -> Option <Span> {
    token.0.ok().map(|token| token.span)
}

macro_rules! define {
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

macro_rules! precedence {
    (
        NEXT = $next:ident,
        'unop: $( $un_name:ident [$($un_fn_debug:ident $un_fn_enum:ident $un_fn:ident )*] )*
        'binop: $( $bin_name:ident [$($bin_fn_debug:ident $bin_fn_enum:ident $bin_fn:ident )*] )*
    ) => {
        #[derive(Copy, Clone, Eq, PartialEq)]
        #[repr(u8)]
        pub enum UnaryOperator {
            $( $( $un_fn_enum, )* )*
        }

        impl Debug for UnaryOperator {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                f.write_str(match self {$($(
                    Self::$un_fn_enum => stringify!($un_fn_debug),
                )*)*})
            }
        }

        #[derive(Copy, Clone, Eq, PartialEq)]
        #[repr(u8)]
        pub enum BinaryOperator {
            $( $( $bin_fn_enum, )* )*
        }

         impl Debug for BinaryOperator {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                f.write_str(match self {$($(
                    Self::$bin_fn_enum => stringify!($bin_fn_debug),
                )*)*})
            }
        }

        precedence!(@rev(@un) [$( ($un_name { $($un_fn_debug $un_fn_enum $un_fn )* }) )* ($next)] | []);
        precedence!(@rev(@bin) [$( ($bin_name { $($bin_fn_debug $bin_fn_enum $bin_fn )* }) )* ($( $un_name )*)] | []);

        precedence!(@expr $( $bin_name )*);
    };

    (@bin ($( $un_name:ident )*) ($name:ident { $( $fun:tt )* }) ) => {
        precedence!(@bin-define $name ($( $un_name )*) $( $fun )*);
    };

    (@bin ($( $un_name:ident )*) ($name:ident { $( $fun:ident )* }) ($next_name:ident { $( $next_fun:tt )* }) $( $tail:tt )*) => {
        precedence!(@bin-define $name ($next_name) $( $fun )*);
        precedence!(@bin ($( $un_name )*) ($next_name { $( $next_fun )* }) $( $tail )*);
    };

    (@bin-define $name:ident ($sub:ident) $( $debug:ident $enumm:ident $fun:ident )*) => {
        define!($name = BinOp <$sub <'code>>, $sub <'code>);

        impl <'code> $name <'code> {
            fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
                let (left, left_ty) = $sub::parse(input, ctx)?;

                let saved_cur_token_for_possible_early_return_triggered_on_inequality_of_spaces_around_the_operator = input.get();

                Result(Ok(match precedence!(@sign-fun(BinaryOperator) input, $( $fun $enumm )*) {
                    Some((span, operator)) => {
                        let (right, right_ty) = $sub::parse(input, ctx)?;

                        let are_left_and_operator_close = (span.start.column - left.span().end.column) == 1;
                        let are_right_and_operator_close = (right.span().start.column - span.end.column) == 1;
                        if are_left_and_operator_close != are_right_and_operator_close {
                            input.set(saved_cur_token_for_possible_early_return_triggered_on_inequality_of_spaces_around_the_operator);
                            return Result(Ok((Self::Partial(Box::new(left)), left_ty)))
                        }

                        let result_ty = match left_ty.perform_binary_operation(input, operator, &right_ty) {
                            Some(x) => x,
                            None => return Result(Err(Error {
                                span,
                                message: format!("cannot {operator:?} the `{:?}` and `{:?}` types", left_ty.debug(input), right_ty.debug(input)),
                                clarifying: format!("incompatible operator and operands"),
                                filename: input.filename.to_string(),
                                code: input.code.to_string()
                            }))
                        };

                        (Self::Full(Box::new(BinOp {
                            left,
                            right,
                            operator: span
                        })), result_ty)
                    },
                    None => (Self::Partial(Box::new(left)), left_ty)
                }))
            }
        }
    };

    (@bin-define $name:ident ($head:ident $( $tail:ident )+) $( $fun:tt )*) => {
        precedence!(@bin-define $name ($( $tail )+) $( $fun )*);
    };

    (@un ($next:ident) ($name:ident { $( $debug:ident $enumm:ident $fun:ident )* })) => {
        define!($name = UnOp <$next <'code>>, $next <'code>);

        impl <'code> $name <'code> {
            fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
                Result(Ok(match precedence!(@sign-fun(UnaryOperator) input, $( $fun $enumm )*) {
                    Some((span, operator)) => {
                        let (value, ty) = $next::parse(input, ctx)?;

                        let result_ty = match ty.perform_unary_operation(input, operator) {
                            Some(x) => x,
                            None => return Result(Err(Error {
                                span,
                                message: format!("cannot {operator:?} the `{:?}` type", ty.debug(input)),
                                clarifying: format!("incompatible operator and operand"),
                                filename: input.filename.to_string(),
                                code: input.code.to_string()
                            }))
                        };

                        (Self::Full(Box::new(UnOp {
                            value,
                            operator: span
                        })), result_ty)
                    },
                    None => {
                        let (operand, ty) = $next::parse(input, ctx)?;
                        (Self::Partial(Box::new(operand)), ty)
                    }
                }))
            }
        }
    };

    // (@un ($name:ident [$( $fun:ident )*]) $( $tail:tt )+) => {
    // TODO
    // };

    (@sign-fun($prefix:ident) $input:ident, $single:ident $operator:ident) => {
        unspan($input.$single()).map(|x| (x, $prefix::$operator))
    };

    (@sign-fun($prefix:ident) $input:ident, $head:ident $operator:ident $( $tail:ident )+) => {
        precedence!(@sign-fun($prefix) $input, $head $operator).or_else(|| precedence!(@sign-fun($prefix) $input, $( $tail )+))
    };

    (@rev($( $label:tt )*) [] | [$( $rev:tt )*]) => {
        precedence! { $( $label )* $( $rev )* }
    };

    (@rev($( $label:tt )*) [( $( $head:tt )* ) $( ( $( $tail:tt )* ) )*] | [$( $rev:tt )*]) => {
        precedence! { @rev($( $label )*) [$( ( $( $tail )* ) )*] | [( $( $head )* ) $( $rev )*] }
    };

    (@expr $last:ident) => {
        type LastExpr <'code> = $last <'code>;

        #[derive(Clone)]
        pub struct Expr <'code> {
            pub value: LastExpr <'code>,
            pub ty: TypeIndex
        }

        impl <'code> GetSpan for Expr <'code> {
            fn span(&self) -> Span {
                self.value.span()
            }
        }

        impl <'code> ParseDebug for Expr <'code> {
            fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
                f.debug_struct("Expr")
                    .field("value", &self.value.debug(input))
                    .field("ty", &self.ty.debug(input))
                    .finish()
            }
        }

        impl <'code> Expr <'code> {
            pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <Self> {
                let (value, ty) = $last::parse(input, ctx)?;

                Result(Ok(Self {
                    value,
                    ty
                }))
            }
        }
    };

    (@expr $head:ident $( $tail:ident )+) => {
        precedence!(@expr $( $tail )+);
    };
}

precedence! {
    NEXT = CallExpr,

    'unop:
        PlusMinusExpr [pos Pos plus negate Neg minus]

    'binop:
        MulDivExpr [multiply Mul star divide Div slash]
        SumDiffExpr [add Add plus sub Sub minus]
}

#[derive(Clone)]
pub struct CallExprFull <'code> {
    /// The index of a function in `ParseInput.top_level_items`
    pub fun: usize,
    pub span: Span,
    pub args: Vec <Expr <'code>>
}

impl <'code> GetSpan for CallExprFull <'code> {
    fn span(&self) -> Span {
        self.span
    }
}

impl <'code> CallExprFull <'code> {
    pub fn get_fun <'a> (&self, input: &'a ParseInput <'code>) -> &'a Fn <'code> {
        match &input.top_level_items[self.fun] {
            Item::Fn(fun) => fun,
            _ => unimplemented!()
        }
    }
}

impl <'code> ParseDebug for CallExprFull <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter<'_>) -> FmtResult {
        struct Inner <'code, 'a> {
            input: &'a ParseInput <'code>,
            args: &'a [Expr <'code>]
        }

        impl <'code, 'a> Debug for Inner <'code, 'a> {
            fn fmt(&self, f: &mut Formatter <'_>) -> FmtResult {
                f.debug_list()
                    .entries(self.args.iter().map(|x| x.debug(&self.input)))
                    .finish()
            }
        }

        f.debug_struct("CallExprFull")
            .field("fun", &self.get_fun(input).name)
            .field("args", &Inner { input, args: &self.args })
            .finish()
    }
}

define!(CallExpr = CallExprFull <'code>, PrimitiveExpr <'code>);

impl <'code> CallExpr <'code> {
    fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        let cur = input.get();

        let ident = match input.ident_as_spanned_str().0 {
            Ok(ok) => ok,
            Err(_) => {
                let (expr, ty) = PrimitiveExpr::parse(input, ctx)?;
                return Result(Ok((Self::Partial(Box::new(expr)), ty)))
            }
        };

        let mut fun_idx = None;
        let mut fun_args_len = None;
        let mut fun_addr = None;

        if let Some(x) = ctx.functions().find(|(_, fun)| fun.name == ident) {
            fun_idx = Some(x.0);
            fun_args_len = Some(x.1.args.len());
            fun_addr = Some(x.1 as *const Fn);
        }

        if fun_idx.is_none() {
            input.set(cur);

            let (expr, ty) = match PrimitiveExpr::parse(input, ctx).0 {
                Ok(ok) => ok,
                _ => return Result(Err(Error {
                    span: ident.span,
                    message: format!("`{}` is not a function", ident.data),
                    clarifying: String::from("here"),
                    filename: input.filename.to_string(),
                    code: input.code.to_string()
                }))
            };

            return Result(Ok((Self::Partial(Box::new(expr)), ty)));
        }

        let fun_idx = unsafe { fun_idx.unwrap_unchecked() };
        let fun_args_len = unsafe { fun_args_len.unwrap_unchecked() };
        let fun_addr = unsafe { fun_addr.unwrap_unchecked() };

        let mut args = vec![];

        let ctx_for_exprs = ctx.add_function_call_deep();

        while !input.is_exhausted() {
            // Let non-top-level function parse only their arguments number,
            // Propagate everything to it otherwise
            if ctx.function_call_deep() != 0 {
                if args.len() == fun_args_len {
                    break
                }
            }

            // SAFETY: exhaustiveness check was performed earlier
            if unsafe { check_if_the_next_token_is_newline(input) } {
                break
            }

            let expr = match Expr::parse(input, &ctx_for_exprs).0 {
                Ok(ok) => ok,
                Err(err) => {
                    input.set(cur);
                    return Result(Err(err))
                }
            };

            args.push(expr)
        }

        let fun = unsafe { &*fun_addr };

        if args.len() != fun_args_len {
            input.set(cur);
            return Result(Err(Error {
                span: ident.span,
                message: String::from("wrong number of arguments"),
                clarifying: format!("expected `{}`, got `{}`", fun_args_len, args.len()),
                filename: input.filename.to_string(),
                code: input.code.to_string()
            }))
        }

        for (parsed, native) in args.iter().zip(&fun.args) {
            if parsed.ty != native.ty {
                return Result(Err(Error {
                    span: parsed.span(),
                    message: String::from("wrong type of the argument"),
                    clarifying: format!("expected `{:?}`, got `{:?}`", native.ty.debug(input), parsed.ty.debug(input)),
                    filename: input.filename.to_string(),
                    code: input.code.to_string()
                }))
            }
        }

        Result(Ok((Self::Full(Box::new(CallExprFull {
            fun: fun_idx,
            span: Span {
                start: ident.span.start,
                end: args.last().map(|x| x.span().end).unwrap_or(ident.span.end)
            },
            args
        })), fun.ret_ty.clone())))
    }
}

#[derive(Clone)]
pub struct BracedExpr <'code> {
    pub value: LastExpr <'code>
}

impl <'code> GetSpan for BracedExpr <'code> {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl <'code> ParseDebug for BracedExpr <'code> {
    #[inline(always)]
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        self.value.debug_impl(input, f)
    }
}

impl <'code> BracedExpr <'code> {
    fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        input.open_brace()?;
        let Expr { value, ty } = Expr::parse(input, ctx)?;
        input.close_brace()?;
        Result(Ok((Self {
            value
        }, ty)))
    }
}

#[derive(Clone)]
pub enum PrimitiveExpr <'code> {
    Ident(Spanned <&'code str>),
    Braced(Box <BracedExpr <'code>>)
}

impl <'code> GetSpan for PrimitiveExpr <'code> {
    fn span(&self) -> Span {
        match self {
            Self::Ident(ident) => ident.span,
            Self::Braced(braced) => braced.span()
        }
    }
}

impl <'code> ParseDebug for PrimitiveExpr <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str("PrimitiveExpr::")?;
        match self {
            Self::Ident(ident) => f.debug_tuple("Ident")
                .field(ident)
                .finish(),
            Self::Braced(expr) => f.debug_tuple("Braced")
                .field(&expr.debug(input))
                .finish()
        }
    }
}

impl <'code> PrimitiveExpr <'code> {
    fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
        if let Result(Ok(ident)) = input.ident_as_spanned_str() {
            Result(if let Some(var) = ctx.variables().find(|v| v.name == ident) {
                Ok((Self::Ident(ident), var.ty.clone()))
            } else {
                Err(Error {
                    span: ident.span,
                    message: format!("no variable named `{}` found", ident.data),
                    clarifying: String::from("here"),
                    filename: input.filename.to_string(),
                    code: input.code.to_string()
                })
            })
        } else {
            let (braced, ty) = BracedExpr::parse(input, ctx)?;
            Result(Ok((Self::Braced(Box::new(braced)), ty)))
        }
    }
}
