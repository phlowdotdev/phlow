use phlow_engine::Context;
use phlow_runtime::{PhlowBuilder, PhlowModule, PhlowModuleSchema, PhlowRuntime};
use phlow_sdk::prelude::{JsonMode, Value, json};
use phlow_sdk::structs::ModuleResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build an in-memory pipeline that uses an inline module.
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

    // Seed the runtime with a main context value.
    let context = Context::from_main(json!({
        "name": "Phlow"
    }));

    // Configure a builder and register an inline module.
    let mut builder = PhlowBuilder::new();
    builder.settings_mut().download = false;
    let mut module = PhlowModule::new();
    // Describe the inline module schema for UI/validation metadata.
    module.set_schema(
        PhlowModuleSchema::new()
            .with_input(json!({ "name": "string" }))
            .with_output(json!({ "message": "string" }))
            .with_input_order(vec!["name"]),
    );
    // Implement the async handler that returns a ModuleResponse.
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

    // Build a prepared runtime from the builder and run it multiple times.
    let mut inline_runtime = builder
        .set_pipeline(pipeline)
        .set_context(context)
        .set_module("inline_echo", module)
        .build()
        .await?;

    // First execution reuses the same prepared runtime.
    let first = inline_runtime.run().await?;
    println!("{}", first.to_json(JsonMode::Inline));

    // Second execution uses the same runtime without rebuilding.
    let second = inline_runtime.run().await?;
    println!("{}", second.to_json(JsonMode::Inline));
    // Release resources after reusing the runtime.
    inline_runtime.shutdown().await?;

    // Example: preprocess a script string once and execute it without reprocessing.
    let script = r#"
steps:
  - return: "ok"
"#;

    // Preprocess the script into a Value using runtime settings and base path.
    let mut preprocess_runtime = PhlowRuntime::new();
    preprocess_runtime.settings_mut().download = false;
    let preprocessed = preprocess_runtime.preprocess_string(script)?;
    // Set the preprocessed value directly, skipping another preprocess step.
    preprocess_runtime.set_preprocessed_pipeline(preprocessed);
    preprocess_runtime.set_context(Context::new());
    // Execute the preprocessed pipeline and print the output.
    let output = preprocess_runtime.run().await?;
    println!("{}", output.to_json(JsonMode::Inline));
    // Shut down the runtime to flush resources.
    preprocess_runtime.shutdown().await?;

    Ok(())
}
