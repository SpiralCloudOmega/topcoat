use crate::{Expr, Interpreter};

/// A handler closure `|p1, p2, ...| body`. The body is required to have unit
/// output — handler bodies are statements, not value expressions. Server-side
/// `eval` is unreachable.
pub struct ExprClosure<Body> {
    params: &'static [&'static str],
    body: Body,
}

impl<Body> ExprClosure<Body> {
    pub fn new(params: &'static [&'static str], body: Body) -> Self {
        Self { params, body }
    }
}

impl<Body> Expr for ExprClosure<Body>
where
    Body: Expr<Output = ()>,
{
    type Output = ();

    fn eval(self, _interpreter: &mut Interpreter) -> Self::Output {
        unreachable!("ExprClosure::eval called server-side; handlers do not run during SSR")
    }

    fn to_js(&self, out: &mut String) {
        out.push_str("((");
        for (i, name) in self.params.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(name);
        }
        out.push_str(") => { ");
        self.body.to_js(out);
        out.push_str("; })");
    }
}
