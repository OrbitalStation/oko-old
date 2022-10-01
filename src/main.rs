use oko::*;

fn main() -> Result <()> {
    let code = std::fs::read_to_string("code")
        .expect("failed to find `code`")
        // Make 4-space combinations be a tab
        .replace("    ", "\t");

    let stream = TokenStream::new("code", &code)?;

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

    println!("{input:#?}");

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
            RawTypeBase::Stub(name) => if let Some(builtin) = BUILTIN_BAKED_TYPES.into_iter().find(|ty| ty.name == name) {
                builtin
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
    'outer: loop {
        // Removes newlines before the item parsing
        loop {
            if input.is_exhausted() {
                break 'outer
            }

            // SAFETY: out-of-bounds case checked above
            let next = unsafe { input.stream.buf.get_unchecked(input.peek("").0.ok().unwrap_unchecked()) };

            if next.kind == TokenKind::Newline {
                input.go_forward()
            } else {
                break
            }
        }

        if input.is_exhausted() {
            break
        }

        let item = Item::parse(input)?;

        input.top_level_items.push(item)
    }

    Result(Ok(()))
}
