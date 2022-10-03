use crate::*;
use core::fmt::{Display, Formatter, Result as FmtResult};

///
/// Pointer to the type(either raw or baked)
///
#[derive(Eq, PartialEq, Clone)]
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
            TypeBaseContainer::Raw(raw) => Display::fmt(&raw[i].name().data, f),
            TypeBaseContainer::Baked(baked) => Display::fmt(&baked[i].name.data, f)
        }
    }
}
