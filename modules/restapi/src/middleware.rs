use hyper::{body::Incoming, service::Service, Request};
use sdk::tracing::{info_span, Instrument};
use std::{future::Future, pin::Pin};
#[derive(Debug, Clone)]
pub struct TracingMiddleware<S> {
    inner: S,
}

impl<S> TracingMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<Request<Incoming>> for TracingMiddleware<S>
where
    S: Service<Request<Incoming>> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        let span = info_span!("http_request", %method, %path);
        let fut = self.inner.call(req);

        Box::pin(async move { fut.instrument(span).await })
    }
}
