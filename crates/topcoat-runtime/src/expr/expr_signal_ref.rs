use std::fmt::Write;

use crate::{Expr, ExprDerefAssignTarget, ExprDerefTarget, Interpreter, IntoExpr, Signal};

pub struct ExprSignalRef<'a, T> {
    signal: &'a Signal<T>,
}

impl<'a, T> Expr for ExprSignalRef<'a, T> {
    type Output = &'a Signal<T>;

    fn eval(self, _interpreter: &mut Interpreter) -> Self::Output {
        self.signal
    }

    fn to_js(&self, out: &mut String) {
        let id =
            serde_json::to_string(&self.signal.id()).expect("signal id is serializable as JSON");
        write!(out, "__context.signal({id})").unwrap();
    }
}

impl<'a, T> IntoExpr for &'a Signal<T> {
    type Expr = ExprSignalRef<'a, T>;

    fn into_expr(self) -> Self::Expr {
        ExprSignalRef { signal: self }
    }
}

impl<'a, T> ExprDerefTarget for &'a Signal<T> {
    type Target = &'a T;

    fn expr_deref(self) -> Self::Target {
        self.read()
    }
}

impl<T> ExprDerefAssignTarget for &Signal<T> {
    type Value = T;

    fn expr_deref_assign(self, _value: T) {
        unreachable!(
            "ExprDerefAssignTarget::expr_deref_assign called server-side; handler bodies do not run during SSR"
        )
    }
}
