use crate::*;

impl <'code> Fn <'code> {
    pub fn get_context <'fun, 'items> (&'fun self, items: &'items [Item <'code>]) -> FnContext <'fun, 'items, 'code> {
        FnContext {
            fun: self,
            items,
            is_primary_call: true
        }
    }
}

pub struct FnContext <'fun, 'items, 'code> {
    fun: &'fun Fn <'code>,
    items: &'items [Item <'code>],
    is_primary_call: bool
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

    fn is_primary_call(&self) -> bool {
        self.is_primary_call
    }

    fn set_not_primary(&self) -> Self {
        Self {
            fun: self.fun,
            items: self.items,
            is_primary_call: false
        }
    }
}
