use phlow_engine::Context;
use phlow_runtime::{PhlowBuilder, PhlowModule, PhlowModuleSchema};
use phlow_sdk::prelude::{json, JsonMode, Value};
use phlow_sdk::structs::ModuleResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline: Value = json!({
        "modules": [
            {
                "module": "inline_echo",
                "name": "inline_echo",
                "with": { "prefix": "Hello, " }
            }
        ],
        "steps": [
            {
                "use": "inline_echo",
                "input": { "name": "{{ main.name }}" }
            },
            { "payload": "{{ payload.message }}" }
        ]
    });

    let context = Context::from_main(json!({
        "name": "Phlow"
    }));

    let mut builder = PhlowBuilder::new();
    builder.settings_mut().download = false;
    let mut module = PhlowModule::new();
    module.set_schema(
        PhlowModuleSchema::new()
            .with_input(json!({ "name": "string" }))
            .with_output(json!({ "message": "string" }))
            .with_input_order(vec!["name"]),
    );
    module.set_handler(|request| async move {
        let name = request
            .input
            .and_then(|value| value.get("name").cloned())
            .unwrap_or_else(|| json!("unknown"));
        let prefix = request
            .with
            .get("prefix")
            .cloned()
            .unwrap_or_else(|| json!(""));
        let message = format!("{}{}", prefix, name);
        ModuleResponse::from_success(json!({ "message": message }))
    });
    let mut runtime = builder
        .set_pipeline(pipeline)
        .set_context(context)
        .set_module("inline_echo", module)
        .build()
        .await?;

    let first = runtime.run().await?;
    println!("{}", first.to_json(JsonMode::Inline));

    let second = runtime.run().await?;
    println!("{}", second.to_json(JsonMode::Inline));
    runtime.shutdown().await?;

    Ok(())
}
