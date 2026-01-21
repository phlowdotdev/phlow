use phlow_engine::Context;
use phlow_runtime::PhlowRuntime;
use phlow_sdk::prelude::Value;

#[test]
fn preprocess_string_inserts_modules() {
    let runtime = PhlowRuntime::new();
    let script = r#"
steps:
  - return: "ok"
"#;

    let value = runtime.preprocess_string(script).unwrap();
    assert!(value.get("steps").is_some());
    let modules = value.get("modules").expect("modules key should exist");
    assert!(modules.is_array());
}

#[tokio::test]
async fn run_preprocessed_pipeline_returns_value() {
    let runtime = PhlowRuntime::new();
    let script = r#"
steps:
  - return: "ok"
"#;

    let pipeline = runtime.preprocess_string(script).unwrap();
    let result = PhlowRuntime::run_preprocessed(pipeline, Context::new())
        .await
        .unwrap();

    assert_eq!(result, Value::from("ok"));
}
