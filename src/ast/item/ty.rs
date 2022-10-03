use core::fmt::{Debug, Formatter, Result as FmtResult};
use std::fmt::Write;
use crate::*;

macro_rules! builtin_types {
    ($vis:vis const $name:ident = [$( ( $($expr:tt)* ) )*]) => {
        $vis const $name: [BuiltinType; builtin_types!(@count $( ($($expr)*) )*)] = [builtin_types!(@iter [0] $( ( $($expr)* ) )*)];
    };

    (@iter [$idx:expr] ($name:literal category: numerik)) => {
        BuiltinType::new($idx, $name, &[
            BuiltinUnaryOperation::new(UnaryOperator::Pos),
            BuiltinUnaryOperation::new(UnaryOperator::Neg)
        ],
        &[
            BuiltinBinaryOperation::new(BinaryOperator::Mul),
            BuiltinBinaryOperation::new(BinaryOperator::Div),
            BuiltinBinaryOperation::new(BinaryOperator::Add),
            BuiltinBinaryOperation::new(BinaryOperator::Sub),
        ])
    };

    (@iter [$idx:expr] ($( $head:tt )*) $( $tail:tt )+) => {
        builtin_types!(@iter [$idx] ($( $head )*)),
        builtin_types!(@iter [$idx + 1] $( $tail )+)
    };

    (@count ($( $tt:tt )*)) => {
        1
    };

    (@count ($( $head:tt )*) $( $tail:tt )+) => {
        builtin_types!(@ $head) + builtin_types!(@ $( $tail )*)
    };
}

builtin_types!(pub const BUILTIN_BAKED_TYPES = [
    ("i32" category: numerik)
]);

pub struct BuiltinType {
    pub base: BakedTypeBase <'static>,
    pub unary_operations: &'static [BuiltinUnaryOperation],
    pub binary_operations: &'static [BuiltinBinaryOperation]
}

impl BuiltinType {
    pub const fn new(
        idx: usize,
        name: &'static str,
        unary_operations: &'static [BuiltinUnaryOperation],
        binary_operations: &'static [BuiltinBinaryOperation]
    ) -> Self {
        Self {
            base: BakedTypeBase::builtin(idx, name),
            unary_operations,
            binary_operations
        }
    }
}

pub struct BuiltinUnaryOperation {
    pub op: UnaryOperator
}

impl BuiltinUnaryOperation {
    pub const fn new(op: UnaryOperator) -> Self {
        Self {
            op
        }
    }
}

pub struct BuiltinBinaryOperation {
    pub op: BinaryOperator
}

impl BuiltinBinaryOperation {
    pub const fn new(op: BinaryOperator) -> Self {
        Self {
            op
        }
    }
}

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
    pub base_index: u32
}

impl TypeIndex {
    /// # Safety
    /// Call only after the baking of all the types
    pub fn perform_unary_operation(&self, input: &ParseInput, op: UnaryOperator) -> Option <TypeIndex> {
        if let BakedTypeBaseKind::Builtin(idx) = self.baked(input).kind {
            BUILTIN_BAKED_TYPES[idx].unary_operations.iter().find(|o| o.op == op).map(|_| self.clone())
        } else {
            None
        }
    }

    /// # Safety
    /// Call only after the baking of all the types
    pub fn perform_binary_operation(&self, input: &ParseInput, op: BinaryOperator, operand: &TypeIndex) -> Option <TypeIndex> {
        let operand = if let BakedTypeBaseKind::Builtin(idx) = operand.baked(input).kind {
            idx
        } else {
            return None
        };

        if let BakedTypeBaseKind::Builtin(idx) = self.baked(input).kind {
            BUILTIN_BAKED_TYPES[idx].binary_operations.iter().find(|o| o.op == op && idx == operand).map(|_| self.clone())
        } else {
            None
        }
    }

    pub fn baked <'a> (&'a self, input: &'a ParseInput) -> &'a BakedTypeBase {
        match &input.type_bases {
            TypeBaseContainer::Baked(baked) => &baked[self.base_index as usize],
            _ => unimplemented!()
        }
    }
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
    Builtin(usize),
    TypeProduct(Punctuated <'code, TypedVariable <'code>, "\n\t">)
}

impl <'code> ParseDebug for BakedTypeBaseKind <'code> {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        f.write_str("BakedTypeBaseKind::")?;

        match self {
            Self::Builtin(_) => f.write_str("Builtin"),
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
    pub const fn builtin(idx: usize, name: &'code str) -> Self {
        Self {
            kind: BakedTypeBaseKind::Builtin(idx),
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
