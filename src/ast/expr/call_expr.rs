use core::fmt::{Debug, Formatter, Result as FmtResult};
use crate::*;

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

define_expr!(CallExpr = CallExprFull <'code>, PrimitiveExpr <'code>);

impl <'code> CallExpr <'code> {
    pub fn parse(input: &mut ParseInput <'code>, ctx: &impl Context <'code>) -> Result <(Self, TypeIndex)> {
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
