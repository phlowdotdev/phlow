use crate::{settings::AuthorizationSpanMode, router::Router, openapi::OpenAPIValidator};
use hyper::{body::Incoming, service::Service, Request};
use phlow_sdk::{
    prelude::*,
    tracing::{field, Dispatch, Level},
};

use std::{future::Future, pin::Pin};

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub id: ModuleId,
    pub sender: MainRuntimeSender,
    pub dispatch: Dispatch,
    pub span: phlow_sdk::tracing::Span,
    pub client_ip: String,
    pub authorization_span_mode: AuthorizationSpanMode,
    pub router: Router,
    pub openapi_validator: Option<OpenAPIValidator>,
    pub cors: Option<crate::setup::CorsConfig>,
}

#[derive(Debug, Clone)]
pub struct TracingMiddleware<S> {
    pub id: usize,
    pub inner: S,
    pub dispatch: phlow_sdk::tracing::Dispatch,
    pub sender: MainRuntimeSender,
    pub peer_addr: std::net::SocketAddr,
    pub authorization_span_mode: AuthorizationSpanMode,
    pub router: Router,
    pub openapi_validator: Option<OpenAPIValidator>,
    pub cors: Option<crate::setup::CorsConfig>,
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
        if req.method() == hyper::Method::GET && req.uri().path() == "/health" {
            let fut: <S as Service<Request<Incoming>>>::Future = self.inner.call(req);
            return Box::pin(async move { fut.await });
        }

        phlow_sdk::tracing::dispatcher::with_default(&self.dispatch.clone(), || {
            let span = tracing::span!(
                Level::INFO,
                "http_request",
                otel.name = field::Empty,
                http.route = field::Empty,
                http.request.body.size = field::Empty,
                http.request.method = field::Empty,
                http.request.size = field::Empty,
                http.connection.state = field::Empty,
                "http.request.method-original" = field::Empty,
                "http.request.resend-count" = field::Empty,
                "http.response.body.size" = field::Empty,
                "http.response.status-code" = field::Empty,
                "http.response.size" = field::Empty,
                "http.request.header.x-real-ip" = self.peer_addr.to_string(),
                "http.request.header.user-agent" = field::Empty,
                "http.request.header.host" = field::Empty,
                "http.request.header.x-request-id" = field::Empty,
                "http.request.header.x-transaction-id" = field::Empty, // custom header
                "http.request.header.x-trace-id" = field::Empty,       // custom header
                "http.request.header.referer" = field::Empty,
                "http.request.header.content-type" = field::Empty,
                "http.request.header.accept" = field::Empty,
                "http.request.header.origin" = field::Empty,
                "http.request.header.x-forwarded-for" = field::Empty,
                "http.request.header.cache-control" = field::Empty,
                "http.request.header.accept-encoding" = field::Empty,
                "http.request.header.authorization" = field::Empty, // Obscure this header
                "http.request.header.accept-language" = field::Empty,
                "http.request.header.connection" = field::Empty,
                "http.request.header.cookie" = field::Empty,
                "http.request.header.if-modified-since" = field::Empty,
                "http.request.header.if-none-match" = field::Empty,
                "http.request.header.content-length" = field::Empty,
                "http.request.header.transfer-encoding" = field::Empty,
                "http.request.header.te" = field::Empty,
                "http.request.header.upgrade-insecure-requests" = field::Empty,
                "http.request.header.sec-fetch-site" = field::Empty,
                "http.request.header.sec-fetch-mode" = field::Empty,
                "http.request.header.sec-fetch-user" = field::Empty,
                "http.request.header.sec-fetch-dest" = field::Empty,
                "http.request.header.dnt" = field::Empty,
                "http.request.header.via" = field::Empty,
                "http.response.header.user-agent" = field::Empty,
                "http.response.header.host" = field::Empty,
                "http.response.header.x-request-id" = field::Empty,
                "http.response.header.x-transaction-id" = field::Empty, // custom header
                "http.response.header.x-trace-id" = field::Empty,       // custom header
                "http.response.header.referer" = field::Empty,
                "http.response.header.content-type" = field::Empty,
                "http.response.header.accept" = field::Empty,
                "http.response.header.origin" = field::Empty,
                "http.response.header.x-forwarded-for" = field::Empty,
                "http.response.header.cache-control" = field::Empty,
                "http.response.header.accept-encoding" = field::Empty,
                "http.response.header.authorization" = field::Empty, // Obscure this header
                "http.response.header.accept-language" = field::Empty,
                "http.response.header.connection" = field::Empty,
                "http.response.header.cookie" = field::Empty,
                "http.response.header.if-modified-since" = field::Empty,
                "http.response.header.if-none-match" = field::Empty,
                "http.response.header.content-length" = field::Empty,
                "http.response.header.transfer-encoding" = field::Empty,
                "http.response.header.te" = field::Empty,
                "http.response.header.upgrade-insecure-requests" = field::Empty,
                "http.response.header.sec-fetch-site" = field::Empty,
                "http.response.header.sec-fetch-mode" = field::Empty,
                "http.response.header.sec-fetch-user" = field::Empty,
                "http.response.header.sec-fetch-dest" = field::Empty,
                "http.response.header.dnt" = field::Empty,
                "http.response.header.via" = field::Empty,
            );

            span_enter!(span);

            let context = RequestContext {
                id: self.id,
                sender: self.sender.clone(),
                dispatch: self.dispatch.clone(),
                client_ip: self.peer_addr.to_string(),
                span,
                authorization_span_mode: self.authorization_span_mode.clone(),
                router: self.router.clone(),
                openapi_validator: self.openapi_validator.clone(),
                cors: self.cors.clone(),
            };

            req.extensions_mut().insert(context);

            let fut: <S as Service<Request<Incoming>>>::Future = self.inner.call(req);

            Box::pin(async move { fut.await })
        })
    }
}
