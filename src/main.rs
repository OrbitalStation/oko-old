use oko::*;

fn main() -> Result <()> {
    let code = std::fs::read_to_string("code")
        .expect("failed to find `code`")
        // Make 4-space combinations be a tab
        .replace("    ", "\t");

    let stream = TokenStream::new("code", &code)?;
    let stream = TokenStream::from(&stream);

    let mut input = ParseInput {
        stream,
        code: &code,
        filename: "code",
        type_bases: TypeBaseContainer::new(),
        fn_body_bases: FnBodyContainer::new(),
        top_level_items: vec![]
    };
    
    parse_code(&mut input)?;

    bake_raw_types(&mut input)?;

    bake_fn_bodies(&mut input)?;

    println!("{input:#?}");

    Result(Ok(()))
}

fn bake_fn_bodies(input: &mut ParseInput) -> Result <()> {
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

        if last.ty != fun.ret_ty {
            return Result(Err(Error {
                span: last.span(),
                message: String::from("return type mismatch"),
                clarifying: format!("expected `{:?}`, got `{:?}`", fun.ret_ty.debug(input), last.ty.debug(input)),
                filename: input.filename.to_string(),
                code: input.code.to_string()
            }))
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

fn bake_raw_types(input: &mut ParseInput) -> Result <()> {
    let raw = match &mut input.type_bases {
        TypeBaseContainer::Raw(raw) => core::mem::replace(raw, vec![]),
        _ => unreachable!()
    };

    let mut newly_baked_types = vec![];

    for base in raw {
        newly_baked_types.push(match base {
            RawTypeBase::Backed(RawTypeDefinition { fields, name }) => BakedTypeBase {
                kind: BakedTypeBaseKind::TypeProduct(fields),
                name
            },
            RawTypeBase::Stub(name) => if let Some(builtin) = BUILTIN_BAKED_TYPES.into_iter().find(|ty| ty.base.name == name) {
                builtin.base
            } else {
                return Result(Err(Error {
                    span: name.span,
                    message: format!("the type `{}` has no definition", name.data),
                    clarifying: String::from("a ghostly type"),
                    filename: input.filename.to_string(),
                    code: input.code.to_string()
                }))
            }
        })
    }

    input.type_bases = TypeBaseContainer::Baked(newly_baked_types);

    Result(Ok(()))
}

fn parse_code(input: &mut ParseInput) -> Result <()> {
    loop {
        remove_newlines(input);

        if input.is_exhausted() {
            break
        }

        let item = Item::parse(input)?;

        input.top_level_items.push(item)
    }

    Result(Ok(()))
}
