use crate::*;
use core::fmt::{Display, Formatter, Result as FmtResult};

///
/// Pointer to the type(either raw or baked)
///
#[derive(Eq, PartialEq, Clone)]
pub enum TypeIndex {
    /// An ordinary type -- `i32`, `bool`, `myCustomType`
    Scalar(u32),

    /// A tuple type -- `(i32, bool)`, `(myi32,)`, `(aType, bType, (cType, dType))`
    Tuple(Vec <TypeIndex>)
}

impl TypeIndex {
    /// # Safety
    /// Call only after the baking of all the types
    pub fn perform_unary_operation(&self, input: &ParseInput, op: UnaryOperator) -> Option <TypeIndex> {
        if let BakedTypeBaseKind::Builtin(idx) = self.baked_scalar(input)?.kind {
            BUILTIN_BAKED_TYPES[idx].unary_operations.iter().find(|o| o.op == op).map(|_| self.clone())
        } else {
            None
        }
    }

    /// # Safety
    /// Call only after the baking of all the types
    pub fn perform_binary_operation(&self, input: &ParseInput, op: BinaryOperator, operand: &TypeIndex) -> Option <TypeIndex> {
        let operand = if let BakedTypeBaseKind::Builtin(idx) = operand.baked_scalar(input)?.kind {
            idx
        } else {
            return None
        };

        if let BakedTypeBaseKind::Builtin(idx) = self.baked_scalar(input)?.kind {
            BUILTIN_BAKED_TYPES[idx].binary_operations.iter().find(|o| o.op == op && idx == operand).map(|_| self.clone())
        } else {
            None
        }
    }

    pub fn baked_scalar <'a> (&'a self, input: &'a ParseInput) -> Option <&'a BakedTypeBase> {
        Some(match &input.type_bases {
            TypeBaseContainer::Baked(baked) => &baked[match self {
                Self::Scalar(idx) => *idx as usize,
                _ => return None
            }],
            _ => unimplemented!()
        })
    }
}

impl <'code> Parse <'code> for TypeIndex {
    fn parse_impl(input: &mut ParseInput <'code>) -> Result <Self> {
        // Try to parse tuple
        if input.open_brace().0.is_ok() {
            let types = Punctuated::<_, "">::new(input, ParseInput::comma, ParseInput::close_brace)?;
            return Result(Ok(Self::Tuple(types.vec)))
        }

        // Fallback to scalars
        let name = input.ident_as_spanned_str()?;

        let base_index = input.find_or_add_raw_type_base(name);

        Result(Ok(Self::Scalar(base_index)))
    }
}

impl ParseDebug for TypeIndex {
    fn debug_impl(&self, input: &ParseInput, f: &mut Formatter <'_>) -> FmtResult {
        match self {
            Self::Scalar(idx) => match &input.type_bases {
                TypeBaseContainer::Raw(raw) => Display::fmt(&raw[*idx as usize].name().data, f),
                TypeBaseContainer::Baked(baked) => Display::fmt(&baked[*idx as usize].name.data, f)
            },
            Self::Tuple(tuple) => if !tuple.is_empty() {
                let mut builder = f.debug_tuple("");
                for ty in tuple {
                    builder.field(&ty.debug(input));
                }
                builder.finish()
            } else {
                f.write_str("()")
            }
        }

    }
}
