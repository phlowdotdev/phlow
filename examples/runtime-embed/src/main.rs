use phlow_engine::Context;
use phlow_runtime::{PhlowBuilder, PhlowModule, PhlowModuleSchema, PhlowRuntime};
use phlow_sdk::prelude::{JsonMode, Value, json};
use phlow_sdk::structs::ModuleResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline: Value = json!({
        "modules": [
            {
                "module": "inline_echo",
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

    let mut inline_runtime = builder
        .set_pipeline(pipeline)
        .set_context(context)
        .set_module("inline_echo", module)
        .build()
        .await?;

    let first = inline_runtime.run().await?;
    println!("{}", first.to_json(JsonMode::Inline));

    let second = inline_runtime.run().await?;
    println!("{}", second.to_json(JsonMode::Inline));
    inline_runtime.shutdown().await?;

    let script = r#"
steps:
  - return: "ok"
"#;

    let mut preprocess_runtime = PhlowRuntime::new();
    preprocess_runtime.settings_mut().download = false;
    let preprocessed = preprocess_runtime.preprocess_string(script)?;
    preprocess_runtime.set_preprocessed_pipeline(preprocessed);
    preprocess_runtime.set_context(Context::new());
    let output = preprocess_runtime.run().await?;
    println!("{}", output.to_json(JsonMode::Inline));
    preprocess_runtime.shutdown().await?;

    Ok(())
}
