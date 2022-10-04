use crate::*;

pub trait Context <'code>: Sized {
	type VariablesIter <'a>: Iterator <Item = &'a TypedVariable <'code>> where 'code: 'a, Self: 'a;

	type FunctionsIter <'a>: Iterator <Item = (usize, &'a Fn <'code>)> where 'code: 'a, Self: 'a;

	/// Returns the iterator of variables that are available to be used at the moment
	fn variables <'a> (&'a self) -> Self::VariablesIter <'a>;

	/// Returns the iterator of functions that are available to be used at the moment
	fn functions <'a> (&'a self) -> Self::FunctionsIter <'a>;

	/// Returns whether this expression is a  primary fn
	fn is_primary_call(&self) -> bool;

	/// Returns self with the change that [`is_primary_call`] should now
	///
	/// return` false`
	fn set_not_primary(&self) -> Self;
}
