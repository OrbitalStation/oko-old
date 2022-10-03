use crate::*;

pub trait Context <'code>: Sized {
	type VariablesIter <'a>: Iterator <Item = &'a TypedVariable <'code>> where 'code: 'a, Self: 'a;

	fn variables <'a> (&'a self) -> Self::VariablesIter <'a>;
}
