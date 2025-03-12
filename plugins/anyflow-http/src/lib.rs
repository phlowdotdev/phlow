use axum::{
    extract::{path, State},
    response::Json,
    routing::{connect, delete, get, head, options, patch, post, put, trace, trace_service},
    Router,
};
use sdk::prelude::*;
use std::{option, sync::Arc};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

#[derive(Clone, FromValue)]
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

#[derive(Clone, FromValue)]
struct Route {
    path: String,
    method: Method,
}

#[derive(Clone, FromValue)]
struct Setup {
    route: Route,
    port: Option<u16>,
    address: Option<String>,
}

#[no_mangle]
pub extern "C" fn process_data(setup: *const Value) {
    unsafe {
        if setup.is_null() {
            return;
        }
        let setup = match Setup::from_value((&*setup).clone()) {
            Some(setup) => Arc::new(setup),
            None => return,
        };

        let rt = Runtime::new().unwrap();

        rt.block_on(start_server(setup));
    }
}

async fn start_server(setup: Arc<Setup>) {
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

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
