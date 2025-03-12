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

// Estado compartilhado para armazenar o callback
#[derive(Clone)]
struct PluginState {
    callback: CallbackFn,
}

#[no_mangle]
pub extern "C" fn process_data(data: *const Value, callback: CallbackFn) {
    unsafe {
        if data.is_null() {
            return;
        }

        let data_ref = &*data;
        println!("🔌 Plugin iniciado com config: {:?}", data_ref);

        let plugin_state = Arc::new(PluginState { callback });

        // Criar e rodar o servidor HTTP dentro de um runtime async
        let rt = Runtime::new().unwrap();
        rt.block_on(start_server(plugin_state));
    }
}

// Função assíncrona que inicia o servidor Axum
async fn start_server(plugin_state: Arc<PluginState>) {
    let app = Router::new()
        .route("/hello", get(hello_handler))
        .route("/ping", get(ping_handler))
        .route("/data", post(data_handler))
        .with_state(plugin_state.clone());

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
