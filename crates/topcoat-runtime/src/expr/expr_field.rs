use std::marker::PhantomData;

use crate::{Expr, Interpreter};

/// A `receiver.field` access on a handler-internal value. The accessor closure
/// passed to `new` exists purely so rustc resolves `T` from the receiver's
/// real type — it is never invoked. Server-side `eval` is unreachable.
pub struct ExprField<R, T> {
    receiver: R,
    name: &'static str,
    _phantom: PhantomData<fn() -> T>,
}

impl<R, T> ExprField<R, T>
where
    R: Expr,
{
    pub fn new<F>(receiver: R, name: &'static str, _accessor: F) -> Self
    where
        F: FnOnce(R::Output) -> T,
    {
        Self {
            receiver,
            name,
            _phantom: PhantomData,
        }
    }
}

impl<R, T> Expr for ExprField<R, T>
where
    R: Expr,
{
    type Output = T;

    fn eval(self, _interpreter: &mut Interpreter) -> Self::Output {
        unreachable!("ExprField::eval called server-side; handler bodies do not run during SSR")
    }

    fn to_js(&self, out: &mut String) {
        out.push('(');
        self.receiver.to_js(out);
        out.push_str(").");
        out.push_str(self.name);
    }
}
