use hyper::body::Body;
use hyper::{body::Incoming, service::Service, Request};
use sdk::tracing::field::FieldSet;
use sdk::tracing::{field, Dispatch, Level, Value};
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
}

use std::{future::Future, pin::Pin};

use crate::trace::META;
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

            let fields = META.fields();

            let val = [
                (
                    &fields.field("otel.name").unwrap(),
                    Some(&span_name as &dyn Value),
                ),
                (&fields.field("http.request.method").unwrap(), Some(&method)),
                (
                    &fields.field("http.request.body.size").unwrap(),
                    Some(&size),
                ),
                (&fields.field("http.route").unwrap(), Some(&path)),
            ];

            let values = fields.value_set(&val);

            // Criando o span manualmente
            let span = tracing::Span::new(&META, &values);

            let context = RequestContext {
                id: self.id,
                sender: self.sender.clone(),
                dispatch: self.dispatch.clone(),
                span: span.clone(),
                client_ip: self.peer_addr.to_string(),
                method: method.clone(),
                path: path.clone(),
            };

            req.extensions_mut().insert(context);

            let fut: <S as Service<Request<Incoming>>>::Future = self.inner.call(req);

            Box::pin(async move { fut.await })
        })
    }
}
