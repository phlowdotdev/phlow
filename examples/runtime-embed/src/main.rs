use phlow_engine::Context;
use phlow_runtime::PhlowBuilder;
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

    let mut builder = PhlowBuilder::new();
    builder.settings_mut().download = false;
    let mut runtime = builder
        .set_pipeline(pipeline)
        .set_context(context)
        .build()
        .await?;

    let first = runtime.run().await?;
    println!("{}", first.to_json(JsonMode::Inline));

    let second = runtime.run().await?;
    println!("{}", second.to_json(JsonMode::Inline));
    runtime.shutdown().await?;

    Ok(())
}
