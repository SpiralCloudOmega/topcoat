//! Fallbacks modeled as errors.
//!
//! A handler can short-circuit by returning one of the fallback errors in
//! this module as the `Err` variant of its [`Result`]. [`redirect`],
//! [`redirect_permanent`], and [`not_found`] construct one directly, and
//! [`FallbackExt`] lets `Option` and `Result` values fall through to a
//! fallback via the `?` operator.

use axum::response::Redirect;
use http::StatusCode;

use crate::Result;

/// Builds a temporary (HTTP 307) redirect to `uri`.
///
/// # Examples
///
/// ```rust,ignore
/// use topcoat::context::Cx;
/// use topcoat::router::{Result, redirect};
///
/// async fn fetch_user(cx: &Cx, id: u64) -> Result<User> {
///     let Some(user) = lookup(cx, id).await else {
///         return Err(redirect("/users").into());
///     };
///     Ok(user)
/// }
/// ```
pub fn redirect(uri: &str) -> RedirectError {
    RedirectError::new(Redirect::temporary(uri))
}

/// Builds a permanent (HTTP 308) redirect to `uri`.
///
/// Use this for URLs that have moved for good — clients and search engines
/// are allowed to cache the new location.
pub fn redirect_permanent(uri: &str) -> RedirectError {
    RedirectError::new(Redirect::permanent(uri))
}

/// Builds a not-found (HTTP 404) response.
///
/// # Examples
///
/// ```rust,ignore
/// use topcoat::context::Cx;
/// use topcoat::router::{Result, not_found};
///
/// async fn fetch_user(cx: &Cx, id: u64) -> Result<User> {
///     let Some(user) = lookup(cx, id).await else {
///         return Err(not_found().into());
///     };
///     Ok(user)
/// }
/// ```
pub fn not_found() -> NotFoundError {
    NotFoundError::new()
}

/// A redirect response carried as the `Err` variant of a handler [`Result`].
///
/// Construct one with [`redirect`] or [`redirect_permanent`], or surface one
/// from an `Option` / `Result` via [`FallbackExt`].
#[derive(Debug)]
pub struct RedirectError {
    inner: axum::response::Redirect,
}

impl RedirectError {
    fn new(inner: axum::response::Redirect) -> Self {
        Self { inner }
    }
}

impl axum::response::IntoResponse for RedirectError {
    fn into_response(self) -> axum::response::Response {
        self.inner.into_response()
    }
}

/// A not-found response carried as the `Err` variant of a handler [`Result`].
///
/// Construct one with [`not_found`], or surface one from an `Option` /
/// `Result` via [`FallbackExt`].
#[derive(Debug)]
pub struct NotFoundError {
    _priv: (),
}

impl NotFoundError {
    fn new() -> Self {
        Self { _priv: () }
    }
}

impl axum::response::IntoResponse for NotFoundError {
    fn into_response(self) -> axum::response::Response {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// Converts an absent or failed value into a fallback response.
///
/// Implemented for [`Option`] (where `None` becomes the fallback) and
/// [`Result`] (where any `Err` becomes the fallback, discarding the
/// original error). Designed to be combined with `?` so a caller can fall
/// through to a redirect or not-found on missing or invalid state.
///
/// # Examples
///
/// ```rust,ignore
/// use topcoat::context::Cx;
/// use topcoat::router::{Result, FallbackExt};
///
/// async fn fetch_user(cx: &Cx, id: u64) -> Result<User> {
///     let user = lookup(cx, id).await.ok_or_redirect("/users")?;
///     Ok(user)
/// }
/// ```
pub trait FallbackExt {
    /// The success type produced when the value is present.
    type T;

    /// Returns `Ok(value)` if present, otherwise a temporary redirect to `uri`.
    fn ok_or_redirect(self, uri: &str) -> Result<Self::T>;

    /// Returns `Ok(value)` if present, otherwise a permanent redirect to `uri`.
    fn ok_or_redirect_permanent(self, uri: &str) -> Result<Self::T>;

    /// Returns `Ok(value)` if present, otherwise a not-found response.
    fn ok_or_not_found(self) -> Result<Self::T>;
}

impl<T> FallbackExt for Option<T> {
    type T = T;

    fn ok_or_redirect(self, uri: &str) -> Result<Self::T> {
        match self {
            Some(value) => Ok(value),
            None => Err(redirect(uri).into()),
        }
    }

    fn ok_or_redirect_permanent(self, uri: &str) -> Result<Self::T> {
        match self {
            Some(value) => Ok(value),
            None => Err(redirect_permanent(uri).into()),
        }
    }

    fn ok_or_not_found(self) -> Result<Self::T> {
        match self {
            Some(value) => Ok(value),
            None => Err(not_found().into()),
        }
    }
}

impl<T, E> FallbackExt for Result<T, E> {
    type T = T;

    fn ok_or_redirect(self, uri: &str) -> Result<Self::T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(redirect(uri).into()),
        }
    }

    fn ok_or_redirect_permanent(self, uri: &str) -> Result<Self::T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(redirect_permanent(uri).into()),
        }
    }

    fn ok_or_not_found(self) -> Result<Self::T> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(not_found().into()),
        }
    }
}
