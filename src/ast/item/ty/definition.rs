use crate::*;
use core::fmt::{Debug, Formatter, Write, Result as FmtResult};

#[derive(Clone)]
pub struct RawTypeDefinition <'code> {
    pub name: Spanned <&'code str>,
    pub fields: Punctuated <'code, TypedVariable <'code>, "\n\t">
}

impl <'code> ParseDebug for RawTypeDefinition <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str("ty ")?;
        self.name.fmt(f)?;
        f.write_char('\n')?;
        self.fields.debug_impl(input, f)?;
        f.write_char('\n')?;

        Ok(())
    }
}

impl <'code> Parse <'code> for RawTypeDefinition <'code> {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
		input.keyword("ty")?;
        let name = input.ident_as_spanned_str()?;

        input.newline()?;

        let block = input.find_end_of_block_and_return_everything_in_it_and_also_go_forward_to_its_end(1);
        let block = unsafe { block.as_ref() };

        let mut fields = Vec::with_capacity(block.len());

        let old_token_stream = core::mem::replace(&mut input.stream, TokenStream::empty());

        input.stream = TokenStream::from(block);

        while !input.is_exhausted() {
            remove_newlines_and_tabs(input);

            if input.is_exhausted() {
                break
            }

            let set = match TypedVariable::parse(input).0 {
                Ok(ok) => ok,
                Err(err) =>  {
                    input.stream = old_token_stream;
                    return Result(Err(err))
                }
            };

            fields.extend(set)
        }

        input.stream = old_token_stream;

        let fields = Punctuated::wrap(fields);

        Result(Ok(Self {
            name,
            fields
        }))
    }
}

/// Pointer to the type definition(i.e. "ty TYPE = ...")
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RawTypeDefinitionIndex {
    idx: usize
}

impl ParseDebug for RawTypeDefinitionIndex {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        let i = self.idx as usize;

        match &input.type_bases {
            TypeBaseContainer::Raw(raw) => raw[i].debug_impl(input, f),
            TypeBaseContainer::Baked(baked) => baked[i].debug_impl(input, f)
        }
    }
}

impl <'code> Parse <'code> for RawTypeDefinitionIndex {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let def = RawTypeDefinition::parse(input)?;

        let idx = match &mut input.type_bases {
            TypeBaseContainer::Raw(raw) => if let Some((idx, base)) = raw
                .iter_mut()
                .enumerate()
                .find(|(_, x)| x.name() == def.name) {
                if let RawTypeBase::Backed(base) = base {
                    return Result(Err(Error {
                        span: def.name.span,
                        message: format!("duplicating type definitions: `{}` at {:?}...", base.name.data, base.name.span.start),
                        clarifying: format!("...and now `{}` at {:?}", def.name.data, def.name.span.start),
                        filename: input.filename.to_string(),
                        code: input.code.to_string()
                    }))
                } else {
                    *base = RawTypeBase::Backed(def);
                    idx
                }
            } else {
                let idx = raw.len();
                raw.push(RawTypeBase::Backed(def));
                idx
            },
            _ => unreachable!()
        };

        Result(Ok(Self {
            idx
        }))
    }
}
