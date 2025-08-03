use crate::loader::Loader;
use crate::settings::{self, Settings};
use crossbeam::channel;
use log::{debug, error};
use phlow_engine::phs::{build_engine, Script};
use phlow_engine::{Context, Phlow};
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::prelude::json;
use phlow_sdk::structs::{ModulePackage, ModuleSetup, Modules};
use phlow_sdk::valu3::prelude::*;
use phlow_sdk::valu3::value::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TestResult {
    pub index: usize,
    pub passed: bool,
    pub message: String,
    pub describe: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<TestResult>,
}

pub async fn run_tests(
    loader: Loader,
    test_filter: Option<&str>,
    settings: Settings,
) -> Result<TestSummary, String> {
    // Get tests from loader.tests
    let tests = loader
        .tests
        .as_ref()
        .ok_or("No tests found in the phlow file")?;
    let steps = &loader.steps;

    if !tests.is_array() {
        return Err(format!("Tests must be an array, got: {:?}", tests));
    }

    let test_cases = tests.as_array().unwrap();

    // Filter tests if test_filter is provided
    let filtered_tests: Vec<_> = if let Some(filter) = test_filter {
        test_cases
            .values
            .iter()
            .enumerate()
            .filter(|(_, test_case)| {
                if let Some(description) = test_case.get("describe") {
                    let desc_str = description.as_string();
                    return desc_str.contains(filter);
                }
                false
            })
            .collect()
    } else {
        test_cases.values.iter().enumerate().collect()
    };

    let total = filtered_tests.len();

    if total == 0 {
        if let Some(filter) = test_filter {
            println!("⚠️  No tests match filter: '{}'", filter);
        } else {
            println!("⚠️  No tests to run");
        }

        return Ok(TestSummary {
            total: 0,
            passed: 0,
            failed: 0,
            results: Vec::new(),
        });
    }

    if let Some(filter) = test_filter {
        println!(
            "🧪 Running {} test(s) matching '{}' (out of {} total)...",
            total,
            filter,
            test_cases.len()
        );
    } else {
        println!("🧪 Running {} test(s)...", total);
    }
    println!();

    // Load modules following the same pattern as Runtime::run
    let modules = load_modules_like_runtime(&loader, settings)
        .await
        .map_err(|e| format!("Failed to load modules for tests: {}", e))?;

    // Create flow from steps
    let workflow = json!({
        "steps": steps
    });

    let phlow = Phlow::try_from_value(&workflow, Some(modules))
        .map_err(|e| format!("Failed to create phlow: {}", e))?;

    // Run tests
    let mut results = Vec::new();
    let mut passed = 0;

    for (run_index, (_, test_case)) in filtered_tests.iter().enumerate() {
        let test_index = run_index + 1;

        // Extract test description if available
        let test_description = test_case.get("describe").map(|v| v.as_string());

        // Print test header with description
        if let Some(ref desc) = test_description {
            print!("Test {}: {} - ", test_index, desc);
        } else {
            print!("Test {}: ", test_index);
        }

        let result = run_single_test(test_case, &phlow).await;

        match result {
            Ok(msg) => {
                println!("✅ PASSED - {}", msg);
                passed += 1;
                results.push(TestResult {
                    index: test_index,
                    passed: true,
                    message: msg,
                    describe: test_description.clone(),
                });
            }
            Err(msg) => {
                println!("❌ FAILED - {}", msg);
                results.push(TestResult {
                    index: test_index,
                    passed: false,
                    message: msg,
                    describe: test_description.clone(),
                });
            }
        }
    }

    let failed = total - passed;
    println!();
    println!("📊 Test Results:");
    println!("   Total: {}", total);
    println!("   Passed: {} ✅", passed);
    println!("   Failed: {} ❌", failed);

    if failed > 0 {
        println!();
        println!("❌ Some tests failed!");
    } else {
        println!();
        println!("🎉 All tests passed!");
    }

    Ok(TestSummary {
        total,
        passed,
        failed,
        results,
    })
}

async fn run_single_test(test_case: &Value, phlow: &Phlow) -> Result<String, String> {
    // Extract test inputs
    let main_value = test_case.get("main").cloned().unwrap_or(Value::Null);
    let initial_payload = test_case.get("payload").cloned().unwrap_or(Value::Null);

    // Create context with test data
    let mut context = Context::from_main(main_value);

    // Set initial payload if provided
    if !initial_payload.is_null() {
        context = context.add_module_output(initial_payload);
    }

    // Execute the workflow
    let result = phlow
        .execute(&mut context)
        .await
        .map_err(|e| format!("Execution failed: {}", e))?;

    let result = result.unwrap_or(Value::Null);

    // Check assertions
    if let Some(assert_eq_value) = test_case.get("assert_eq") {
        if deep_equals(&result, assert_eq_value) {
            Ok(format!("Expected and got: {}", result))
        } else {
            Err(format!("Expected {}, got {}", assert_eq_value, result))
        }
    } else if let Some(assert_expr) = test_case.get("assert") {
        // For assert expressions, we need to evaluate them
        let assertion_result = evaluate_assertion(assert_expr, &result)
            .map_err(|e| format!("Assertion error: {}", e))?;

        if assertion_result {
            Ok(format!("Assertion passed: {}", assert_expr))
        } else {
            Err(format!("Assertion failed: {}", assert_expr))
        }
    } else {
        Err("No assertion found (assert or assert_eq required)".to_string())
    }
}

// Load modules following the exact same pattern as Runtime::run
// but without creating main_sender channels since we don't need them for tests
async fn load_modules_like_runtime(
    loader: &Loader,
    settings: Settings,
) -> Result<Arc<Modules>, String> {
    let mut modules = Modules::default();

    // Initialize tracing subscriber
    let guard = init_tracing_subscriber(loader.app_data.clone());
    let dispatch = guard.dispatch.clone();

    let engine = build_engine(None);

    // Load modules exactly like Runtime::run does
    for (id, module) in loader.modules.iter().enumerate() {
        let (setup_sender, setup_receive) =
            oneshot::channel::<Option<channel::Sender<ModulePackage>>>();

        // For tests, we never pass main_sender to prevent modules from starting servers/loops
        let main_sender = None;

        let with = {
            let script = Script::try_build(engine.clone(), &module.with)
                .map_err(|e| format!("Failed to build script for module {}: {}", module.name, e))?;

            script.evaluate_without_context().map_err(|e| {
                format!(
                    "Failed to evaluate script for module {}: {}",
                    module.name, e
                )
            })?
        };

        let setup = ModuleSetup {
            id,
            setup_sender,
            main_sender,
            with,
            dispatch: dispatch.clone(),
            app_data: loader.app_data.clone(),
            is_test_mode: true,
        };

        let module_target = module.module.clone();
        let module_version = module.version.clone();
        let is_local_path = module.local_path.is_some();
        let local_path = module.local_path.clone();
        let module_name = module.name.clone();
        let settings = settings.clone();

        debug!(
            "Module debug: name={}, is_local_path={}, local_path={:?}",
            module_name, is_local_path, local_path
        );

        // Load module in separate thread - same as Runtime::run
        std::thread::spawn(move || {
            let result =
                Loader::load_module(setup, &module_target, &module_version, local_path, settings);

            if let Err(err) = result {
                error!("Test runtime Error Load Module: {:?}", err)
            }
        });

        debug!(
            "Module {} loaded with name \"{}\" and version \"{}\"",
            module.module, module.name, module.version
        );

        // Wait for module registration - same as Runtime::run
        match setup_receive.await {
            Ok(Some(sender)) => {
                debug!("Module {} registered", module.name);
                modules.register(module.clone(), sender);
            }
            Ok(None) => {
                debug!("Module {} did not register", module.name);
            }
            Err(_) => {
                return Err(format!("Module {} registration failed", module.name));
            }
        }
    }

    Ok(Arc::new(modules))
}

/// Deep equality comparison for JSON values that ignores object property order
/// and compares structure recursively
fn deep_equals(a: &Value, b: &Value) -> bool {
    match (a, b) {
        // Same type comparisons
        (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // Compare numeric values regardless of internal type representation
            let a_val = a.to_f64().unwrap_or(0.0);
            let b_val = b.to_f64().unwrap_or(0.0);
            (a_val - b_val).abs() < f64::EPSILON
        }
        (Value::String(a), Value::String(b)) => a == b,

        // Array comparison - order matters for arrays
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.values
                .iter()
                .zip(b.values.iter())
                .all(|(a_val, b_val)| deep_equals(a_val, b_val))
        }

        // Object comparison - order doesn't matter for objects
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }

            // Check if all keys from a exist in b with equal values
            for (key, a_val) in a.iter() {
                let key_str = key.to_string();
                match b.get(key_str.as_str()) {
                    Some(b_val) => {
                        if !deep_equals(a_val, b_val) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }

            true
        }

        // Different types are not equal
        _ => false,
    }
}

fn evaluate_assertion(assert_expr: &Value, result: &Value) -> Result<bool, String> {
    // Create a simple evaluation context
    let engine = build_engine(None);

    // Convert the assertion expression to a script
    let script = Script::try_build(engine, assert_expr)
        .map_err(|e| format!("Failed to build assertion script: {}", e))?;

    // Create a context where 'payload' refers to the result
    let _context = Context::from_main(json!({
        "payload": result
    }));

    // Evaluate the assertion
    let context_map: HashMap<String, Value> = [("payload".to_string(), result.clone())]
        .iter()
        .cloned()
        .collect();

    let assertion_result = script
        .evaluate(&context_map)
        .map_err(|e| format!("Failed to evaluate assertion: {}", e))?;

    // Check if result is boolean true
    match assertion_result {
        Value::Boolean(b) => Ok(b),
        Value::String(s) if s == "true".into() => Ok(true),
        Value::String(s) if s == "false".into() => Ok(false),
        _ => Err(format!(
            "Assertion must return boolean, got: {}",
            assertion_result
        )),
    }
}
