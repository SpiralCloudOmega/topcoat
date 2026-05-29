mod eval;
mod expr_assign_deref;
mod expr_closure;
mod expr_field;
// mod expr_method_call;
mod expr_param;
mod expr_raw;
mod expr_value;
mod fmt_js;

pub use eval::*;
pub use expr_assign_deref::*;
pub use expr_closure::*;
pub use expr_field::*;
// pub use expr_method_call::*;
pub use expr_param::*;
pub use expr_raw::*;
pub use expr_value::*;
pub use fmt_js::*;

#[derive(Debug, Clone)]
pub struct Expr<T> {
    value: T,
    js: String,
}

impl<T> Expr<T> {
    #[inline]
    pub fn new(js: String, value: T) -> Self {
        Self { value, js }
    }

    #[inline]
    pub fn js(&self) -> &str {
        &self.js
    }

    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn into_value(self) -> T {
        self.value
    }
}

pub trait ToExpr {
    type Expr;

    fn to_expr(value: &Self) -> Self::Expr;
}

impl<T> ToExpr for T
where
    T: Eval,
{
    type Expr = ExprValue<T>;

    fn to_expr(_value: &Self) -> Self::Expr {
        ExprValue::new(std::marker::PhantomData)
    }
}
