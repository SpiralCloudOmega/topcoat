use axum::body::Body;
use http::Request;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct Cx<'a> {}

#[derive(Clone)]
struct CxLayer;

#[derive(Clone)]
struct CxMiddleware<S> {
    inner: S,
}

impl<S> Layer<S> for CxLayer {
    type Service = CxMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CxMiddleware { inner }
    }
}

impl<S> Service<Request<Body>> for CxMiddleware<S>
where
    S: Service<Request<Body>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Self::Response = future.await?;
            Ok(response)
        })
    }
}
