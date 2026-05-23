use crate::{Expr, Interpreter};

pub trait ExprDerefTarget {
    type Target;

    fn expr_deref(self) -> Self::Target;
}

pub struct ExprDeref<E>(E);

impl<E> ExprDeref<E> {
    pub fn new(inner: E) -> Self {
        Self(inner)
    }
}

impl<E> Expr for ExprDeref<E>
where
    E: Expr,
    E::Output: ExprDerefTarget,
{
    type Output = <E::Output as ExprDerefTarget>::Target;

    fn eval(self, interpreter: &mut Interpreter) -> Self::Output {
        self.0.eval(interpreter).expr_deref()
    }

    fn to_js(&self, out: &mut String) {
        // In JS, maverick signal handles are callable; reading is `handle()`.
        out.push('(');
        self.0.to_js(out);
        out.push_str(")()");
    }
}
