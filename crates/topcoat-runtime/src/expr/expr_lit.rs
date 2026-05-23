use serde::Serialize;

use crate::{Expr, Interpreter, IntoExpr};

pub struct ExprLit<T>(T);

impl<T> Expr for ExprLit<T>
where
    T: Serialize,
{
    type Output = T;

    fn eval(self, _interpreter: &mut Interpreter) -> Self::Output {
        self.0
    }

    fn to_js(&self, out: &mut String) {
        let json = serde_json::to_string(&self.0).expect("literal is serializable as JSON");
        out.push_str(&json);
    }
}

macro_rules! impl_primitive {
    ($ty:ty) => {
        impl IntoExpr for $ty {
            type Expr = ExprLit<Self>;

            fn into_expr(self) -> Self::Expr {
                ExprLit(self)
            }
        }
    };
}

impl_primitive!(bool);
impl_primitive!(f64);
impl_primitive!(&'static str);
