use crate::route::RouteId;

#[derive(Debug, Default, Clone)]
pub(crate) struct DynamicRoutes {
    router: matchit::Router<RouteId>,
}

impl DynamicRoutes {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, pattern: impl Into<String>, route_id: RouteId) {
        self.router.insert(pattern.into(), route_id);
    }

    pub fn get<'path>(&mut self, path: &'path str) -> Option<DynamicMatch<'_, 'path>> {
        self.router
            .at(path)
            .ok()
            .map(|result| DynamicMatch::new(*result.value, result.params))
    }
}

pub(crate) struct DynamicMatch<'k, 'v> {
    route_id: RouteId,
    params: matchit::Params<'k, 'v>,
}

impl<'k, 'v> DynamicMatch<'k, 'v> {
    fn new(route_id: RouteId, params: matchit::Params<'k, 'v>) -> Self {
        Self { route_id, params }
    }
}
