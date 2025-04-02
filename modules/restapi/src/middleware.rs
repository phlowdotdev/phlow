use hyper::body::Body;
use hyper::{body::Incoming, service::Service, Request};
use sdk::tracing::field::FieldSet;
use sdk::tracing::{field, span, Dispatch, Level, Value};
use sdk::tracing::{Metadata, Span};
use sdk::tracing_core::callsite::{DefaultCallsite, Identifier};
use sdk::tracing_core::{self, Callsite, Kind};
use sdk::ModuleId;
use sdk::{tracing, MainRuntimeSender};

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub id: ModuleId,
    pub sender: MainRuntimeSender,
    pub dispatch: Dispatch,
    pub span: sdk::tracing::Span,
    pub client_ip: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
}

use std::collections::HashMap;
use std::{future::Future, pin::Pin};

use crate::resolver::resolve_headers;
use crate::trace::get_meta;

#[derive(Debug, Clone)]
pub struct TracingMiddleware<S> {
    pub id: usize,
    pub inner: S,
    pub dispatch: sdk::tracing::Dispatch,
    pub sender: MainRuntimeSender,
    pub peer_addr: std::net::SocketAddr,
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

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
        sdk::tracing::dispatcher::with_default(&self.dispatch.clone(), || {
            let path = req.uri().path().to_string();
            let method = req.method().to_string();
            let span_name = format!("{} {}", method, path);
            let size = req.body().size_hint().lower();

            let headers = resolve_headers(req.headers().clone());
            let span = tracing::span!(
                Level::INFO,
                "http_request",
                otel.name = span_name,
                http.route = path.clone(),
                http.request.body.size = size,
                http.request.method = method.clone(),
                http.request.header.user_agent =
                    headers.get("user-agent").unwrap_or(&"".to_string()),
                http.request.header.host = headers.get("host").unwrap_or(&"".to_string()),
                http.request.header.x_request_id =
                    headers.get("x-request-id").unwrap_or(&"".to_string()),
                http.request.header.x_transaction_id =
                    headers.get("x-transaction-id").unwrap_or(&"".to_string()),
                http.request.header.referer = headers.get("referer").unwrap_or(&"".to_string()),
                http.request.header.content_type =
                    headers.get("content-type").unwrap_or(&"".to_string()),
                http.request.header.accept = headers.get("accept").unwrap_or(&"".to_string()),
                http.request.header.origin = headers.get("origin").unwrap_or(&"".to_string()),
                http.request.header.x_forwarded_for =
                    headers.get("x-forwarded-for").unwrap_or(&"".to_string()),
                http.request.header.x_real_ip = self.peer_addr.to_string(),
                http.request.header.cache_control =
                    headers.get("cache-control").unwrap_or(&"".to_string()),
                http.request.header.accept_encoding =
                    headers.get("accept-encoding").unwrap_or(&"".to_string()),
                http.response.status_code = 0,
                http.response.size = 0
            );

            let context = RequestContext {
                id: self.id,
                sender: self.sender.clone(),
                dispatch: self.dispatch.clone(),
                span: span.clone(),
                client_ip: self.peer_addr.to_string(),
                method: method.clone(),
                path: path.clone(),
                headers: headers.clone(),
            };

            req.extensions_mut().insert(context);

            let fut: <S as Service<Request<Incoming>>>::Future = self.inner.call(req);

            Box::pin(async move { fut.await })
        })
    }
}
