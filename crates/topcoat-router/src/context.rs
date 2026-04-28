use axum::{body::Body, extract::Request};
use tokio::task_local;

#[derive(Debug)]
pub struct Cx {
    request: Request<Body>,
}

task_local! {
    static CX: Cx;
}

pub(crate) async fn scope_context<F: Future>(request: Request<Body>, f: F) -> F::Output {
    CX.scope(Cx { request }, f).await
}

pub async fn with_context<F, R>(f: F) -> R
where
    F: FnOnce(&Cx) -> R,
{
    CX.with(f)
}
