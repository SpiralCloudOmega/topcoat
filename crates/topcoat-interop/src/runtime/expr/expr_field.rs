use std::marker::PhantomData;

use crate::runtime::{Eval, FmtJs, Formatter};

pub struct ExprField<R, T> {
    receiver: R,
    name: &'static str,
    _phantom: PhantomData<fn() -> T>,
}

impl<R, T> ExprField<R, T>
where
    R: Eval,
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

impl<R, T> Eval for ExprField<R, T>
where
    R: Eval,
{
    type Output = T;
}

impl<R, T> FmtJs for ExprField<R, T>
where
    R: FmtJs,
{
    fn fmt_js(&self, f: &mut Formatter<'_>) {
        f.write_char('(');
        self.receiver.fmt_js(f);
        f.write_str(").");
        f.write_str(self.name);
    }
}
