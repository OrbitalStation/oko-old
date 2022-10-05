use crate::*;

pub fn bake_raw_types(input: &mut ParseInput) -> Result <()> {
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
