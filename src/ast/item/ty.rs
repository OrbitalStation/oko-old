use core::fmt::{Debug, Formatter, Result as FmtResult};
use std::fmt::Write;
use crate::*;

const_array!(pub const BUILTIN_BAKED_TYPES: [BakedTypeBase] = [
    BakedTypeBase::builtin("i32")
]);

#[derive(Clone)]
pub struct RawTypeDefinition <'code> {
    pub name: Spanned <&'code str>,
    pub fields: Punctuated <'code, TypedVariablesSet <'code>, "\n\t">
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

        let mut fields = Vec::with_capacity(block.len());

        let old_token_stream = core::mem::replace(&mut input.stream, TokenStream::empty());

        for line in &block {
            input.stream = TokenStream::from(unsafe { line.as_ref().to_vec() });

            let set = match TypedVariablesSet::parse(input).0 {
                Ok(ok) => ok,
                Err(err) =>  {
                    input.stream = old_token_stream;
                    return Result(Err(err))
                }
            };

            fields.push(set)
        }

        input.stream = old_token_stream;
        // let next = &input.stream.buf[input.peek("expected either a `=` or a newline")?];
        // let fields = if next.kind == TokenKind::Eq {
        //     input.go_forward();
        //     Punctuated::single(TypedVariablesSet::parse(input)?)
        // } else if next.kind == TokenKind::Newline {
        //     input.go_forward();
        //     // let block = input.find_end_of_block_and_return_everything_in_it_and_also_go_forward_to_its_end(1).to_vec();
        //     //
        //     // let old_token_stream = core::mem::replace(&mut input.stream, TokenStream::from(block));
        //     //
        //     // match Punctuated::new(input, |input| {
        //     //     input.newline()?;
        //     //     input.tab()
        //     // }, |input| {
        //     //     input.newline()?;
        //     //
        //     //     if input.tab().0.is_ok() {
        //     //         if input.tab().0.is_err() {
        //     //             return Result(Err(Error::STUB))
        //     //         }
        //     //     }
        //     //
        //     //     Result(Ok(&Token::STUB))
        //     // }).0 {
        //     //     Ok(ok) => ok,
        //     //     Err(err) => {
        //     //         input.stream = old_token_stream;
        //     //         return Result(Err(err))
        //     //     }
        //     // }
        // } else {
        //     return input.generate_expected_err("either a `=` or a newline", next)
        // };

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
        let i =  self.idx as usize;

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

#[repr(u8)]
#[derive(Clone)]
pub enum RawTypeBase <'code> {
    Stub(Spanned <&'code str>),
    Backed(RawTypeDefinition <'code>)
}

impl <'code> RawTypeBase <'code> {
    pub const fn name(&self) -> Spanned <&'code str> {
        match self {
            Self::Stub(name) => *name,
            Self::Backed(RawTypeDefinition { name, .. }) => *name
        }
    }
}

impl <'code> ParseDebug for RawTypeBase <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        match self {
            Self::Stub(name) => f.write_fmt(format_args!("RawTypeBaseBackedWithADefinition::Stub({name:?})")),
            Self::Backed(def) => {
                f.write_str("RawTypeBaseBackedWithADefinition::Backed(")?;
                def.debug_impl(input, f)?;
                f.write_char(')')
            }
        }
    }
}

///
/// Pointer to the type(either raw or baked)
///
#[derive(Clone)]
pub struct TypeIndex {
    base_index: u32
}

impl <'code> Parse <'code> for TypeIndex {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        let name = input.ident_as_spanned_str()?;

        let base_index = input.find_or_add_raw_type_base(name);

        Result(Ok(Self {
            base_index
        }))
    }
}

impl ParseDebug for TypeIndex {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        let i = self.base_index as usize;

        match &input.type_bases {
            TypeBaseContainer::Raw(raw) => raw[i].name().fmt(f),
            TypeBaseContainer::Baked(baked) => baked[i].name.fmt(f)
        }
    }
}

#[derive(Clone)]
#[repr(u8)]
pub enum BakedTypeBaseKind <'code> {
    Builtin,
    TypeProduct(Punctuated <'code, TypedVariablesSet <'code>, "\n\t">)
}

impl <'code> ParseDebug for BakedTypeBaseKind <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str("BakedTypeBaseKind::")?;

        match self {
            Self::Builtin => f.write_str("Builtin"),
            Self::TypeProduct(fields) => {
                f.write_str("TypeProduct(")?;
                fields.debug_impl(input, f)?;
                f.write_char(')')
            }
        }?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct BakedTypeBase <'code> {
    pub kind: BakedTypeBaseKind <'code>,
    pub name: Spanned <&'code str>
}

impl <'code> BakedTypeBase <'code> {
    pub const fn builtin(name: &'code str) -> Self {
        Self {
            kind: BakedTypeBaseKind::Builtin,
            name: Spanned {
                data: name,
                span: Span::DEFAULT
            }
        }
    }
}

impl <'code> ParseDebug for BakedTypeBase <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.debug_struct("BakedTypeBase")
            .field("kind", &self.kind.debug(input))
            .field("name", &self.name)
        .finish()
    }
}

#[derive(Clone)]
#[repr(u8)]
pub enum TypeBaseContainer <'code> {
    /// `Raw` is a type just parsed from a file with no extra processing
    /// performed over it
    Raw(Vec <RawTypeBase <'code>>),

    /// `Baked` is a type that went through all the processing and information collecting,
    /// ready to be used and transpiled into the backend
    Baked(Vec <BakedTypeBase <'code>>)
}

impl <'code> ParseDebug for TypeBaseContainer <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        let mut list = f.debug_list();

        match self {
            Self::Raw(raw) => list.entries(raw.iter().map(|x| x.debug(input))),
            Self::Baked(baked) => list.entries(baked.iter().map(|x| x.debug(input))),
        };

        list.finish()
    }
}

impl <'code> TypeBaseContainer <'code> {
    pub const fn new() -> Self {
        Self::Raw(vec![])
    }
}
