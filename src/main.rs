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
