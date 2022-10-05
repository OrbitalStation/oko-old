use crate::*;

pub fn bake_fn_bodies(input: &mut ParseInput) -> Result <()> {
    let raw = match &mut input.fn_body_bases {
        FnBodyContainer::Raw(raw) => core::mem::replace(raw, vec![]),
        _ => unreachable!()
    };

    let mut newly_baked_bodies = vec![];

    let mut exprs = vec![];

    let old_token_stream = core::mem::replace(&mut input.stream, TokenStream::empty());

    let items = core::mem::replace(&mut input.top_level_items, vec![]);

    for (idx, body) in raw.into_iter().enumerate() {
        let tokens = body.body();

        let fun = match items.iter().find(|item| matches!(item, Item::Fn(fun) if fun.body.base_index == idx as u32)).unwrap() {
            Item::Fn(fun) => fun,
            _ => unreachable!()
        };

        let ctx = fun.get_context(&items);

        input.stream = TokenStream::from(tokens);

        while !input.is_exhausted() {
            remove_newlines_and_tabs(input);

            if input.is_exhausted() {
                break
            }

            let expr = Expr::parse(input, &ctx)?;

            exprs.push(expr)
        }

        // Check body non-emptiness
        let last = match exprs.last() {
            Some(x) => x,
            None => return Result(Err(Error {
                span: fun.name.span,
                message: String::from("functions cannot have empty body"),
                clarifying: String::from("help: try using `pass`"),
                filename: input.filename.to_string(),
                code: input.code.to_string()
            }))
        };

        // Check last expr's type and return types are same
        if last.ty != fun.ret_ty {
            return Result(Err(Error {
                span: last.span(),
                message: String::from("return type mismatch"),
                clarifying: format!("expected `{:?}`, got `{:?}`", fun.ret_ty.debug(input), last.ty.debug(input)),
                filename: input.filename.to_string(),
                code: input.code.to_string()
            }))
        }

        // Check all exprs(except the last one) have `()` type
        let mut exprs_iter = exprs.iter();
        exprs_iter.advance_back_by(1).unwrap();
        for expr in exprs_iter {
            if !expr.ty.is_unit_tuple() {
                return Result(Err(Error {
                    span: expr.span(),
                    message: format!("type mismatch: non-return expression should have `()` type, got `{:?}`", expr.ty.debug(input)),
                    clarifying: String::from("help: try using `drop` function"),
                    filename: input.filename.to_string(),
                    code: input.code.to_string()
                }))
            }
        }

        newly_baked_bodies.push(BakedFnBodyBase {
            body: core::mem::replace(&mut exprs, vec![])
        })
    }

    input.stream = old_token_stream;

    input.fn_body_bases = FnBodyContainer::Baked(newly_baked_bodies);

    input.top_level_items = items;

    Result(Ok(()))
}
