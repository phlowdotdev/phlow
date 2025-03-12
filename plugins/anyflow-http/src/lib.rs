use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use sdk::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

#[derive(Clone, FromValue)]
enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Clone, FromValue)]
struct Route {
    path: String,
    method: Method,
}

#[derive(Clone, FromValue)]
struct Setup {
    port: u16,
    routes: Vec<Route>,
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

async fn start_server(plugin_state: Arc<Setup>) {
    let mut app = Router::new();

    for route in plugin_state.routes.iter() {
        match route.method {
            Method::GET => {
                app = app.route(&route.path, get(hello_handler));
            }
            Method::POST => {
                app = app.route(&route.path, post(data_handler));
            }
            _ => {}
        }
    }

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Servidor rodando em http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

// Rota 1: GET /hello
async fn hello_handler() -> Json<String> {
    Json("👋 Olá do plugin HTTP!".to_string())
}

// Rota 2: GET /ping
async fn ping_handler() -> Json<String> {
    Json("🏓 Pong!".to_string())
}

// Rota 3: POST /data → Chama o callback e retorna a resposta
async fn data_handler(State(state): State<Arc<PluginState>>) -> Json<String> {
    let response_value = Value::from("🔄 Plugin chamou o callback!");
    let boxed_value = Box::new(response_value);

    let result_ptr = (state.callback)(Box::into_raw(boxed_value));

    if !result_ptr.is_null() {
        let result_ref = unsafe { &*result_ptr };
        Json(format!("🔧 Callback retornou: {:?}", result_ref))
    } else {
        Json("⚠️ Callback não retornou nada!".to_string())
    }
}
