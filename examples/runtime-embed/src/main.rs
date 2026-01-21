use phlow_engine::Context;
use phlow_runtime::PhlowRuntime;
use phlow_sdk::prelude::{json, JsonMode, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline: Value = json!({
        "steps": [
            { "payload": "{{ main.name }}" },
            { "payload": "{{ \"Hello, \" + payload }}" }
        ]
    });

    let context = Context::from_main(json!({
        "name": "Phlow"
    }));

    let mut runtime = PhlowRuntime::new();
    runtime.set_pipeline(pipeline);
    runtime.set_context(context);
    runtime.settings_mut().download = false;
    runtime.build().await?;

    let result = runtime.run().await?;
    println!("{}", result.to_json(JsonMode::Inline));

    Ok(())
}
