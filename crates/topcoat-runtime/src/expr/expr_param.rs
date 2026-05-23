use std::marker::PhantomData;

use crate::{Expr, Interpreter};

/// References a closure parameter by name. The user-annotated parameter type
/// flows in as `T`, so field accesses against this expression resolve against
/// the real type. Server-side `eval` is unreachable — handlers do not run
/// during SSR.
pub struct ExprParam<T> {
    name: &'static str,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> ExprParam<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: PhantomData,
        }
    }
}

impl<T> Expr for ExprParam<T> {
    type Output = T;

    fn eval(self, _interpreter: &mut Interpreter) -> Self::Output {
        unreachable!("ExprParam::eval called server-side; handler bodies do not run during SSR")
    }

    fn to_js(&self, out: &mut String) {
        out.push_str(self.name);
    }
}
