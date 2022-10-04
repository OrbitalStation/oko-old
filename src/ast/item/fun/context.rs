use crate::*;

impl <'code> Fn <'code> {
    pub fn get_context <'fun, 'items> (&'fun self, items: &'items [Item <'code>]) -> FnContext <'fun, 'items, 'code> {
        FnContext {
            fun: self,
            items,
            function_call_deep: 0
        }
    }
}

pub struct FnContext <'fun, 'items, 'code> {
    fun: &'fun Fn <'code>,
    items: &'items [Item <'code>],
    function_call_deep: usize
}

impl <'fun, 'items, 'code> Context <'code> for FnContext <'fun, 'items, 'code> {
    type VariablesIter <'a> = core::slice::Iter <'a, TypedVariable <'code>> where 'code: 'a, Self: 'a;

    type FunctionsIter <'a> = core::iter::FilterMap <core::iter::Enumerate <core::slice::Iter <'a, Item <'code>>>, for <'b> fn((usize, &'b Item <'code>)) -> Option <(usize, &'b Fn <'code>)>> where 'code: 'a, Self: 'a;

    fn variables <'a> (&'a self) -> Self::VariablesIter <'a> {
        self.fun.args.iter()
    }

    fn functions <'a> (&'a self) -> Self::FunctionsIter <'a> {
        self.items.iter().enumerate().filter_map(|(idx, item)| match item {
            Item::Fn(fun) => Some((idx, fun)),
            _ => None
        })
    }

    fn function_call_deep(&self) -> usize {
        self.function_call_deep
    }

    fn add_function_call_deep(&self) -> Self {
        Self {
            fun: self.fun,
            items: self.items,
            function_call_deep: self.function_call_deep + 1
        }
    }
}
