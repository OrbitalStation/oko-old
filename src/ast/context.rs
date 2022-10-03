use crate::*;

pub trait Context <'code>: Sized {
	type VariablesIter <'a>: Iterator <Item = &'a TypedVariable <'code>> where 'code: 'a, Self: 'a;

	type FunctionsIter <'a>: Iterator <Item = (usize, &'a Fn <'code>)> where 'code: 'a, Self: 'a;

	/// Returns the iterator of variables that are available to be used at the moment
	fn variables <'a> (&'a self) -> Self::VariablesIter <'a>;

	/// Returns the iterator of functions that are available to be used at the moment
	fn functions <'a> (&'a self) -> Self::FunctionsIter <'a>;

	/// Returns how many function calls are parents of the current expression
	fn function_call_deep(&self) -> usize;

	/// Returns self with the change that [`function_call_deep`] should now
	///
	/// return 1 + previous value of it
	fn add_function_call_deep(&self) -> Self;
}
