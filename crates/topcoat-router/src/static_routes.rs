use std::{borrow::Cow, collections::HashMap};

use http::Method;

use crate::route::RouteId;

#[derive(Debug, Default, Clone)]
pub(crate) struct StaticRoutes {
    routes: HashMap<StaticRouteKey<'static>, RouteId>,
}

impl StaticRoutes {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(
        &mut self,
        method: Method,
        path: impl Into<Cow<'static, str>>,
        route_id: RouteId,
    ) {
        self.routes
            .insert(StaticRouteKey::new(method, path.into()), route_id);
    }

    pub fn get(&self, method: Method, path: &str) -> Option<RouteId> {
        self.routes
            .get(&StaticRouteKey::new(method, path.into()))
            .copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StaticRouteKey<'a> {
    method: Method,
    path: Cow<'a, str>,
}

impl<'a> StaticRouteKey<'a> {
    fn new(method: Method, path: Cow<'a, str>) -> Self {
        Self { method, path }
    }
}
