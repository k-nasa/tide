use async_std::future::Future;

use crate::utils::BoxFuture;
use crate::{response::IntoResponse, Request, Response};

/// An HTTP request handler.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are functions that take a `Request<State>` as an argument and
/// return a type `T` that implements [`IntoResponse`].
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```no_run
/// async fn hello(_cx: tide::Request<()>) -> String {
///     String::from("hello")
/// }
///
/// fn main() {
///     let mut app = tide::Server::new();
///     app.at("/hello").get(hello);
/// }
/// ```
///
/// An endpoint with similar functionality that does not make use of the `async` keyword would look something like this:
///
/// ```no_run
/// # use core::future::Future;
/// fn hello(_cx: tide::Request<()>) -> impl Future<Output = String> {
///     futures::future::ready(String::from("hello"))
/// }
///
/// fn main() {
///     let mut app = tide::Server::new();
///     app.at("/hello").get(hello);
/// }
/// ```
///
/// Tide routes will also accept endpoints with `Fn` signatures of this form, but using the `async` keyword has better ergonomics.
pub trait Endpoint<State>: Send + Sync + 'static {
    /// The async result of `call`.
    type Fut: Future<Output = Response> + Send + 'static;

    /// Invoke the endpoint within the given context
    fn call(&self, cx: Request<State>) -> Self::Fut;
}

pub(crate) type DynEndpoint<State> =
    dyn (Fn(Request<State>) -> BoxFuture<'static, Response>) + 'static + Send + Sync;

impl<State, F: Send + Sync + 'static, Fut> Endpoint<State> for F
where
    F: Fn(Request<State>) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse,
{
    type Fut = BoxFuture<'static, Response>;
    fn call(&self, cx: Request<State>) -> Self::Fut {
        let fut = (self)(cx);
        Box::pin(async move { fut.await.into_response() })
    }
}
