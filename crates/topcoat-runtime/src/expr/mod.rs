mod expr_assign_deref;
mod expr_closure;
mod expr_deref;
mod expr_field;
mod expr_lit;
mod expr_method_call;
mod expr_param;
mod expr_signal_ref;
mod interpreter;

pub use expr_assign_deref::*;
pub use expr_closure::*;
pub use expr_deref::*;
pub use expr_field::*;
pub use expr_lit::*;
pub use expr_method_call::*;
pub use expr_param::*;
pub use expr_signal_ref::*;
pub use interpreter::*;

/// A reactive expression. `eval` runs server-side for SSR-time evaluation
/// (only meaningful for read-position nodes); `to_js` emits a JavaScript
/// fragment that the browser compiles via `new Function('__context', …)`.
pub trait Expr {
    type Output;

    fn eval(self, interp: &mut Interpreter) -> Self::Output;
    fn to_js(&self, out: &mut String);
}

pub trait IntoExpr {
    type Expr;

    fn into_expr(self) -> Self::Expr;
}
