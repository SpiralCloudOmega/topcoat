use std::marker::PhantomData;

use crate::runtime::{Eval, ToExpr};

pub struct ExprValue<T>(PhantomData<T>);

impl<T> ExprValue<T> {
    #[inline]
    pub const fn new(inner: PhantomData<T>) -> Self {
        Self(inner)
    }
}

impl<T> Eval for ExprValue<T> {
    type Output = T;
}

impl ToExpr for &'static str {
    type Expr = ExprValue<Self>;

    fn to_expr(value: &Self) -> Self::Expr {
        ExprValue::new(PhantomData)
    }
}
