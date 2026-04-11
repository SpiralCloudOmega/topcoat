use http::Method;

use crate::{Pattern, Route, Routes};

#[derive(Debug, Default, Clone)]
pub struct Router {
    routes: Routes,
}

impl Router {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&mut self, pattern: impl Into<Pattern>) -> &mut Self {
        self.route(Method::GET, pattern.into())
    }

    fn route(&mut self, method: Method, pattern: Pattern) -> &mut Self {
        self.routes.insert(Route::new());
        self
    }
}
