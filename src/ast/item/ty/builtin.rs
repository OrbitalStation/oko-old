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
