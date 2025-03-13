use axum::{
    response::Json,
    routing::{connect, delete, get, head, options, patch, post, put, trace, trace_service},
    Router,
};
use sdk::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone, FromValue, Debug)]
enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    TRACE,
    CONNECT,
    ANY,
}

#[derive(Clone, Debug)]
struct Route {
    path: String,
    method: Method,
}

impl From<Value> for Route {
    fn from(value: Value) -> Self {
        let path = value.get("path").unwrap().as_string();
        let method = value.get("method").unwrap().as_string();

        let method = match method.as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "OPTIONS" => Method::OPTIONS,
            "HEAD" => Method::HEAD,
            "TRACE" => Method::TRACE,
            "CONNECT" => Method::CONNECT,
            _ => Method::ANY,
        };

        Route { path, method }
    }
}

#[derive(Clone, Debug)]
struct Setup {
    route: Route,
    port: Option<u16>,
    address: Option<String>,
}

impl From<Value> for Setup {
    fn from(value: Value) -> Self {
        let route = {
            let route = value.get("route").unwrap().clone();
            Route::from(route)
        };

        let port = match value.get("port") {
            Some(port) => Some(port.to_u64().unwrap() as u16),
            None => None,
        };

        let address = match value.get("address") {
            Some(address) => Some(address.as_string()),
            None => None,
        };

        Setup {
            route,
            port,
            address,
        }
    }
}

plugin_async!(setup);

async fn setup(value: &Value) {
    println!("setup {:?}", value);
    if value.is_null() {
        println!("Value is null");
        return;
    }

    let setup = Arc::new(Setup::from(value.clone()));

    start_server(setup).await;
}

async fn start_server(setup: Arc<Setup>) {
    println!("start_server {:?}", setup);
    async fn handler(Json(payload): Json<Value>) -> Json<String> {
        println!("{:?}", payload);
        Json("Hello, World!".to_string())
    }

    let app: Router = Router::new();
    let app = match setup.route.method {
        Method::GET => app.route(&setup.route.path, get(handler)),
        Method::POST => app.route(&setup.route.path, post(handler)),
        Method::PUT => app.route(&setup.route.path, put(handler)),
        Method::DELETE => app.route(&setup.route.path, delete(handler)),
        Method::PATCH => app.route(&setup.route.path, patch(handler)),
        Method::OPTIONS => app.route(&setup.route.path, options(handler)),
        Method::HEAD => app.route(&setup.route.path, head(handler)),
        Method::TRACE => app.route(&setup.route.path, trace(handler)),
        Method::CONNECT => app.route(&setup.route.path, connect(handler)),
        Method::ANY => app
            .route(&setup.route.path, get(handler))
            .route(&setup.route.path, post(handler))
            .route(&setup.route.path, put(handler))
            .route(&setup.route.path, delete(handler))
            .route(&setup.route.path, patch(handler))
            .route(&setup.route.path, options(handler))
            .route(&setup.route.path, head(handler))
            .route(&setup.route.path, trace(handler))
            .route(&setup.route.path, connect(handler)),
    };

    let address = setup.address.as_deref().unwrap_or("0.0.0.0");
    let port = setup.port.unwrap_or(3000);
    let addr = format!("{}:{}", address, port);

    println!("Listening on: {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
