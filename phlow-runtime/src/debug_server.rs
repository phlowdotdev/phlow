use phlow_engine::debug::{DebugController, DebugReleaseResult, DebugSnapshot};
use phlow_sdk::prelude::{JsonMode, ToValueBehavior, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

const DEFAULT_DEBUG_PORT: u16 = 31400;

pub async fn spawn(controller: Arc<DebugController>) -> std::io::Result<()> {
    let addr = debug_addr();
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Phlow debug server listening on {}", addr);

    tokio::spawn(async move {
        if let Err(err) = serve(listener, controller).await {
            log::error!("Phlow debug server failed: {}", err);
        }
    });

    Ok(())
}

async fn serve(
    listener: TcpListener,
    controller: Arc<DebugController>,
) -> std::io::Result<()> {
    loop {
        let (stream, addr) = listener.accept().await?;
        log::debug!("Debug client connected: {}", addr);
        let controller = controller.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, controller).await {
                log::debug!("Debug client error: {}", err);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    controller: Arc<DebugController>,
) -> std::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }

        let response = match cmd.to_ascii_uppercase().as_str() {
            "STEP" => match controller.current_snapshot().await {
                Some(snapshot) => snapshot_to_value(&snapshot),
                None => error_value("no step waiting"),
            },
            "NEXT" => {
                match controller.release_next().await {
                    DebugReleaseResult::Released => ok_value(),
                    DebugReleaseResult::Awaiting => error_value("awaiting step"),
                    DebugReleaseResult::NoStep => error_value("no step waiting"),
                }
            }
            "ALL" => {
                let history = controller.history().await;
                let values: Vec<Value> = history.iter().map(snapshot_to_value).collect();
                values.to_value()
            }
            "RELEASE" => {
                match controller.release_pipeline().await {
                    DebugReleaseResult::Released => ok_value(),
                    DebugReleaseResult::Awaiting => error_value("awaiting step"),
                    DebugReleaseResult::NoStep => error_value("no step waiting"),
                }
            }
            _ => error_value("unknown command"),
        };

        let payload = response.to_json(JsonMode::Inline);
        writer.write_all(payload.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    Ok(())
}

fn debug_addr() -> String {
    let port = std::env::var("PHLOW_DEBUG_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_DEBUG_PORT);
    format!("0.0.0.0:{}", port)
}

fn snapshot_to_value(snapshot: &DebugSnapshot) -> Value {
    let payload = snapshot.context.payload.clone().unwrap_or(Value::Null);
    let main = snapshot.context.main.clone().unwrap_or(Value::Null);
    let mut context_map = HashMap::new();
    context_map.insert("payload".to_string(), payload);
    context_map.insert("main".to_string(), main);

    let mut map = HashMap::new();
    map.insert("context".to_string(), context_map.to_value());
    map.insert("step".to_string(), snapshot.step.clone());
    map.insert(
        "pipeline".to_string(),
        (snapshot.pipeline as i64).to_value(),
    );

    map.to_value()
}

fn ok_value() -> Value {
    let mut map = HashMap::new();
    map.insert("ok".to_string(), true.to_value());
    map.to_value()
}

fn error_value(message: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("ok".to_string(), false.to_value());
    map.insert("error".to_string(), message.to_value());
    map.to_value()
}
