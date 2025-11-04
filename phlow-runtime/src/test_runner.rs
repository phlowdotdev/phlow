use crate::loader::{Loader, load_module};
use crate::settings::Settings;
use crossbeam::channel;
use log::{debug, error};
use phlow_engine::phs::{self, build_engine};
use phlow_engine::script::Script;
use phlow_engine::{Context, Phlow};
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::prelude::json;
use phlow_sdk::structs::{ModulePackage, ModuleSetup, Modules};
use phlow_sdk::valu3::prelude::*;
use phlow_sdk::valu3::value::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

#[derive(Debug, Clone)]
struct SingleTestReport {
    ok: bool,
    message: String,
    main: Value,
    initial_payload: Value,
    result: Value,
}

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
    debug!("run_tests");
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

    // Helpers to support nested describes/its
    fn is_group(v: &Value) -> bool {
        v.get("tests").map(|t| t.is_array()).unwrap_or(false)
    }

    fn group_name(v: &Value) -> Option<String> {
        v.get("describe")
            .or_else(|| v.get("name"))
            .map(|s| s.as_string())
    }

    fn leaf_title(v: &Value) -> Option<String> {
        v.get("it")
            .or_else(|| v.get("describe"))
            .map(|s| s.as_string())
    }

    fn path_matches_filter(path: &[String], title: &str, filter: &str) -> bool {
        let mut full = path.join(" › ");
        if !full.is_empty() {
            full.push_str(" › ");
        }
        full.push_str(title);
        full.contains(filter)
    }

    fn count_leafs(items: &Value, filter: Option<&str>, ancestors: &Vec<String>) -> usize {
        let mut count = 0usize;
        if let Some(arr) = items.as_array() {
            for item in &arr.values {
                if is_group(item) {
                    let mut new_path = ancestors.clone();
                    if let Some(name) = group_name(item) {
                        new_path.push(name);
                    }
                    count += count_leafs(&item.get("tests").unwrap(), filter, &new_path);
                } else {
                    // leaf
                    let title = leaf_title(item).unwrap_or_else(|| "".to_string());
                    if let Some(f) = filter {
                        if path_matches_filter(ancestors, &title, f) {
                            count += 1;
                        }
                    } else {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    // Count total leaf tests considering the optional filter
    let total = count_leafs(tests, test_filter, &Vec::new());

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

    // Run tests (com suporte a describe aninhado) usando uma lista de ações síncrona
    let mut results = Vec::new();
    let mut passed = 0;
    let mut executed = 0usize;
    // Global "test" map shared across tests
    let tests_global = Arc::new(Mutex::new(json!({})));
    let engine = build_engine(None);

    enum Action {
        Heading {
            name: String,
            depth: usize,
        },
        Test {
            case: Value,
            path: Vec<String>,
            title: String,
            depth: usize,
        },
    }

    fn build_actions(
        items: &Value,
        filter: Option<&str>,
        path: &mut Vec<String>,
        depth: usize,
        out: &mut Vec<Action>,
    ) {
        if let Some(arr) = items.as_array() {
            for item in &arr.values {
                if is_group(item) {
                    let name = group_name(item).unwrap_or_else(|| "(group)".to_string());
                    // Check if any leaf inside matches filter
                    let group_has = {
                        fn inner_count(v: &Value, f: Option<&str>, mut p: Vec<String>) -> usize {
                            let mut c = 0usize;
                            if let Some(name) = group_name(v) {
                                p.push(name);
                            }
                            if let Some(ts) = v.get("tests").and_then(|t| t.as_array()) {
                                for x in &ts.values {
                                    if is_group(x) {
                                        c += inner_count(x, f, p.clone());
                                    } else {
                                        let title = leaf_title(x).unwrap_or_else(|| "".to_string());
                                        if let Some(ff) = f {
                                            let mut s = p.join(" › ");
                                            if !s.is_empty() {
                                                s.push_str(" › ");
                                            }
                                            s.push_str(&title);
                                            if s.contains(ff) {
                                                c += 1;
                                            }
                                        } else {
                                            c += 1;
                                        }
                                    }
                                }
                            }
                            c
                        }
                        inner_count(item, filter, path.clone())
                    };
                    if group_has == 0 {
                        continue;
                    }
                    out.push(Action::Heading {
                        name: name.clone(),
                        depth,
                    });
                    path.push(name);
                    build_actions(&item.get("tests").unwrap(), filter, path, depth + 1, out);
                    path.pop();
                } else {
                    let title = leaf_title(item).unwrap_or_else(|| "(test)".to_string());
                    let mut full = path.join(" › ");
                    if !full.is_empty() {
                        full.push_str(" › ");
                    }
                    full.push_str(&title);
                    if let Some(f) = filter {
                        if !full.contains(f) {
                            continue;
                        }
                    }
                    out.push(Action::Test {
                        case: item.clone(),
                        path: path.clone(),
                        title,
                        depth,
                    });
                }
            }
        }
    }

    let mut actions: Vec<Action> = Vec::new();
    let mut status_map: HashMap<String, bool> = HashMap::new();
    let mut path_stack: Vec<String> = Vec::new();
    build_actions(tests, test_filter, &mut path_stack, 0, &mut actions);

    let mut failed_details: Vec<(String, SingleTestReport)> = Vec::new();

    for action in actions {
        match action {
            Action::Heading { name, depth } => {
                debug!("Test Group: {} (depth {})", name, depth);
            }
            Action::Test {
                case,
                path,
                title,
                depth,
            } => {
                executed += 1;
                let mut full = path.join(" › ");
                if !full.is_empty() {
                    full.push_str(" › ");
                }
                full.push_str(&title);

                let rep =
                    run_single_test(&case, &phlow, tests_global.clone(), engine.clone()).await;
                if rep.ok {
                    debug!("Test Passed: {} (depth {})", full, depth);
                    passed += 1;
                    status_map.insert(full.clone(), true);
                    results.push(TestResult {
                        index: executed,
                        passed: true,
                        message: rep.message.clone(),
                        describe: Some(full.clone()),
                    });
                } else {
                    debug!("Test Failed: {} (depth {})", full, depth);
                    status_map.insert(full.clone(), false);
                    results.push(TestResult {
                        index: executed,
                        passed: false,
                        message: rep.message.clone(),
                        describe: Some(full.clone()),
                    });
                    failed_details.push((full.clone(), rep));
                }
            }
        }
    }

    let failed = executed - passed;
    println!();
    println!("📊 Test Results:");
    println!("   Total: {}", executed);
    println!("   Passed: {} ✅", passed);
    println!("   Failed: {} ❌", failed);

    if failed > 0 {
        println!();
        println!("❌ Some tests failed!");
    } else {
        println!();
        println!("🎉 All tests passed!");
    }

    // Print a final tree view of describes and tests with pass/fail
    {
        fn is_group(v: &Value) -> bool {
            v.get("tests").map(|t| t.is_array()).unwrap_or(false)
        }
        fn group_name(v: &Value) -> Option<String> {
            v.get("describe")
                .or_else(|| v.get("name"))
                .map(|s| s.as_string())
        }
        fn leaf_title(v: &Value) -> Option<String> {
            v.get("it")
                .or_else(|| v.get("describe"))
                .map(|s| s.as_string())
        }

        fn collect_visible_children<'a>(
            value: &'a Value,
            filter: Option<&str>,
            path: &Vec<String>,
        ) -> Vec<&'a Value> {
            let mut out = Vec::new();
            if let Some(arr) = value.as_array() {
                for item in &arr.values {
                    if is_group(item) {
                        let mut p = path.clone();
                        if let Some(n) = group_name(item) {
                            p.push(n);
                        }
                        // if group has any visible child, include it
                        let inner =
                            collect_visible_children(&item.get("tests").unwrap(), filter, &p);
                        if !inner.is_empty() {
                            out.push(item);
                        }
                    } else {
                        let title = leaf_title(item).unwrap_or_else(|| "".to_string());
                        let mut full = path.join(" › ");
                        if !full.is_empty() {
                            full.push_str(" › ");
                        }
                        full.push_str(&title);
                        if let Some(f) = filter {
                            if !full.contains(f) {
                                continue;
                            }
                        }
                        out.push(item);
                    }
                }
            }
            out
        }

        fn print_tree(
            nodes: &Value,
            filter: Option<&str>,
            path: &mut Vec<String>,
            prefix: &str,
            status: &HashMap<String, bool>,
        ) {
            let visible = collect_visible_children(nodes, filter, path);
            let len = visible.len();
            for (idx, node) in visible.into_iter().enumerate() {
                let last = idx + 1 == len;
                let (branch, next_prefix) = if last {
                    ("└── ", format!("{}    ", prefix))
                } else {
                    ("├── ", format!("{}│   ", prefix))
                };
                if is_group(node) {
                    let name = group_name(node).unwrap_or_else(|| "(group)".to_string());
                    println!("{}{}describe: {}", prefix, branch, name);
                    path.push(name);
                    print_tree(
                        &node.get("tests").unwrap(),
                        filter,
                        path,
                        &next_prefix,
                        status,
                    );
                    path.pop();
                } else {
                    let title = leaf_title(node).unwrap_or_else(|| "(test)".to_string());
                    let mut full = path.join(" › ");
                    if !full.is_empty() {
                        full.push_str(" › ");
                    }
                    full.push_str(&title);
                    let icon = match status.get(&full) {
                        Some(true) => "✅",
                        Some(false) => "❌",
                        None => "•",
                    };
                    println!("{}{}{} it: {}", prefix, branch, icon, title);
                }
            }
        }

        println!("\n🌲 Test Tree:");
        let mut p: Vec<String> = Vec::new();
        print_tree(tests, test_filter, &mut p, "", &status_map);
    }

    // Print details for failed tests: inputs and outputs, formatted (in red)
    if failed > 0 {
        // ANSI Red start
        println!("\n\x1b[31m🧾 Failed tests details:");
        for (full_name, rep) in failed_details.iter() {
            println!("\n{}:", full_name);
            // Entrada
            println!("  Entrada:");
            println!("    main: {}", rep.main);
            if !rep.initial_payload.is_undefined() {
                println!("    payload: {}", rep.initial_payload);
            }
            // Saída
            println!("  Saída:");
            println!("    payload: {}", rep.result);
        }
        // ANSI Reset
        println!("\x1b[0m");
    }

    Ok(TestSummary {
        total: executed,
        passed,
        failed,
        results,
    })
}

async fn run_single_test(
    test_case: &Value,
    phlow: &Phlow,
    test: Arc<Mutex<Value>>,
    engine: Arc<phlow_engine::phs::Engine>,
) -> SingleTestReport {
    let tests_snapshot = { test.lock().await.clone() };
    let mut context = Context::from_tests(tests_snapshot.clone());

    // Extract test inputs
    let main_value = {
        let data = test_case.get("main").cloned().unwrap_or(Value::Undefined);

        match Script::try_build(engine.clone(), &Value::from(data)) {
            Ok(script) => match script.evaluate(&context) {
                Ok(val) => val.to_value(),
                Err(e) => {
                    return SingleTestReport {
                        ok: false,
                        message: format!("Failed to evaluate main script: {}", e),
                        main: Value::Undefined,
                        initial_payload: Value::Undefined,
                        result: Value::Undefined,
                    };
                }
            },
            Err(e) => {
                return SingleTestReport {
                    ok: false,
                    message: format!("Failed to build main script: {}", e),
                    main: Value::Undefined,
                    initial_payload: Value::Undefined,
                    result: Value::Undefined,
                };
            }
        }
    };
    let initial_payload = {
        let data = test_case
            .get("payload")
            .cloned()
            .unwrap_or(Value::Undefined);

        if data.is_undefined() {
            Value::Undefined
        } else {
            match Script::try_build(engine.clone(), &Value::from(data)) {
                Ok(script) => match script.evaluate(&context) {
                    Ok(val) => val.to_value(),
                    Err(e) => {
                        return SingleTestReport {
                            ok: false,
                            message: format!("Failed to evaluate payload script: {}", e),
                            main: main_value.clone(),
                            initial_payload: Value::Undefined,
                            result: Value::Undefined,
                        };
                    }
                },
                Err(e) => {
                    return SingleTestReport {
                        ok: false,
                        message: format!("Failed to build payload script: {}", e),
                        main: main_value.clone(),
                        initial_payload: Value::Undefined,
                        result: Value::Undefined,
                    };
                }
            }
        }
    };

    debug!(
        "Running test with main: {:?}, payload: {:?}",
        main_value, initial_payload
    );

    context = context.clone_with_main(main_value.clone());

    // Set initial payload if provided
    if !initial_payload.is_undefined() {
        context = context.clone_with_output(initial_payload.clone());
    }

    // Execute the workflow
    let result = {
        let exec = match phlow.execute(&mut context).await {
            Ok(v) => v,
            Err(e) => {
                return SingleTestReport {
                    ok: false,
                    message: format!("Execution failed: {}", e),
                    main: main_value.clone(),
                    initial_payload: initial_payload.clone(),
                    result: Value::Undefined,
                };
            }
        };
        exec.unwrap_or(Value::Undefined)
    };

    // Check assertions
    // Identify this test id (used to store into global tests)
    let test_id = test_case
        .get("id")
        .map(|v| v.as_string())
        .or_else(|| test_case.get("describe").map(|v| v.as_string()))
        .or_else(|| test_case.get("it").map(|v| v.as_string()))
        .unwrap_or_else(|| "current".to_string());

    if let Some(assert_eq_value) = test_case.get("assert_eq") {
        // ANSI escape code for red: \x1b[31m ... \x1b[0m
        if deep_equals(&result, assert_eq_value) {
            // Update global tests map with this test execution
            {
                let mut guard = test.lock().await;
                let mut map: HashMap<String, Value> = HashMap::new();
                if let Some(obj) = guard.as_object() {
                    for (k, v) in obj.iter() {
                        map.insert(k.to_string(), v.clone());
                    }
                }
                map.insert(
                    test_id.clone(),
                    json!({
                        "id": test_id.clone(),
                        "describe": test_case.get("describe").cloned().unwrap_or(Value::Undefined),
                        "main": main_value.clone(),
                        "payload": result.clone(),
                    }),
                );
                *guard = Value::from(map);
            }

            SingleTestReport {
                ok: true,
                message: format!("Expected and got: {}", result),
                main: main_value.clone(),
                initial_payload: initial_payload.clone(),
                result: result.clone(),
            }
        } else {
            let mut msg = String::new();
            write!(
                &mut msg,
                "Expected \x1b[34m{}\x1b[0m, got \x1b[31m{}\x1b[0m",
                assert_eq_value, result
            )
            .unwrap();
            // Update global tests map with this test execution even on failure
            {
                let mut guard = test.lock().await;
                let mut map: HashMap<String, Value> = HashMap::new();
                if let Some(obj) = guard.as_object() {
                    for (k, v) in obj.iter() {
                        map.insert(k.to_string(), v.clone());
                    }
                }
                map.insert(
                    test_id.clone(),
                    json!({
                        "id": test_id.clone(),
                        "describe": test_case.get("describe").cloned().unwrap_or(Value::Undefined),
                        "main": main_value.clone(),
                        "payload": result.clone(),
                    }),
                );
                *guard = Value::from(map);
            }

            SingleTestReport {
                ok: false,
                message: msg,
                main: main_value.clone(),
                initial_payload: initial_payload.clone(),
                result: result.clone(),
            }
        }
    } else if let Some(assert_expr) = test_case.get("assert") {
        // For assert expressions, we need to evaluate them
        let assertion_result = match evaluate_assertion(
            assert_expr,
            main_value.clone(),
            tests_snapshot,
            result.clone(),
        ) {
            Ok(v) => v,
            Err(e) => {
                return SingleTestReport {
                    ok: false,
                    message: format!("Assertion error: {}. payload: {}", e, result),
                    main: main_value.clone(),
                    initial_payload: initial_payload.clone(),
                    result: result.clone(),
                };
            }
        };

        if assertion_result {
            // Update global tests map with this test execution
            {
                let mut guard = test.lock().await;
                let mut map: HashMap<String, Value> = HashMap::new();
                if let Some(obj) = guard.as_object() {
                    for (k, v) in obj.iter() {
                        map.insert(k.to_string(), v.clone());
                    }
                }
                map.insert(
                    test_id.clone(),
                    json!({
                        "id": test_id.clone(),
                        "describe": test_case.get("describe").cloned().unwrap_or(Value::Undefined),
                        "main": main_value.clone(),
                        "payload": result.clone(),
                    }),
                );
                *guard = Value::from(map);
            }

            SingleTestReport {
                ok: true,
                message: format!("Assertion passed: {}", assert_expr),
                main: main_value.clone(),
                initial_payload: initial_payload.clone(),
                result: result.clone(),
            }
        } else {
            // Print the full payload when an assert fails
            // Update global tests map with this test execution even on failure
            {
                let mut guard = test.lock().await;
                let mut map: HashMap<String, Value> = HashMap::new();
                if let Some(obj) = guard.as_object() {
                    for (k, v) in obj.iter() {
                        map.insert(k.to_string(), v.clone());
                    }
                }
                map.insert(
                    test_id.clone(),
                    json!({
                        "id": test_id.clone(),
                        "describe": test_case.get("describe").cloned().unwrap_or(Value::Undefined),
                        "main": main_value.clone(),
                        "payload": result.clone(),
                    }),
                );
                *guard = Value::from(map);
            }

            SingleTestReport {
                ok: false,
                message: format!(
                    "Assertion failed: {}. payload: \x1b[31m{}\x1b[0m",
                    assert_expr, result
                ),
                main: main_value.clone(),
                initial_payload: initial_payload.clone(),
                result: result.clone(),
            }
        }
    } else {
        // Update global tests map with this test execution even when no assertion is defined
        {
            let mut guard = test.lock().await;
            let mut map: HashMap<String, Value> = HashMap::new();
            if let Some(obj) = guard.as_object() {
                for (k, v) in obj.iter() {
                    map.insert(k.to_string(), v.clone());
                }
            }
            map.insert(
                test_id.clone(),
                json!({
                    "id": test_id.clone(),
                    "describe": test_case.get("describe").cloned().unwrap_or(Value::Undefined),
                    "main": main_value.clone(),
                    "payload": result.clone(),
                }),
            );
            *guard = Value::from(map);
        }

        SingleTestReport {
            ok: false,
            message: "No assertion found (assert or assert_eq required)".to_string(),
            main: main_value.clone(),
            initial_payload: initial_payload.clone(),
            result: result.clone(),
        }
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
            let script = phs::Script::try_build(engine.clone(), &module.with)
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
            let result = load_module(setup, &module_target, &module_version, local_path, settings);

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
                debug!("Module \"{}\" registered", module.name);
                modules.register(module.clone(), sender);
            }
            Ok(None) => {
                debug!("Module \"{}\" did not register", module.name);
            }
            Err(err) => {
                return Err(format!(
                    "Module \"{}\" registration failed: {}",
                    module.name, err
                ));
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

fn evaluate_assertion(
    assert_expr: &Value,
    main: Value,
    tests: Value,
    result: Value,
) -> Result<bool, String> {
    // Create a simple evaluation context
    let engine = build_engine(None);

    // Convert the assertion expression to a script
    let script = Script::try_build(engine, assert_expr)
        .map_err(|e| format!("Failed to build assertion script: {}", e))?;

    // Create a context where 'payload' refers to the result and 'test'/'steps' point to global tests map
    let mut context = Context::from_main_tests(main, tests);

    context.add_step_payload(Some(result));

    match script.evaluate(&context) {
        Ok(Value::Boolean(b)) => Ok(b),
        Ok(Value::String(s)) if s == "true".into() => Ok(true),
        Ok(Value::String(s)) if s == "false".into() => Ok(false),
        Ok(other) => Err(format!("Assertion must return boolean, got: {}", other)),
        Err(e) => Err(format!("Failed to evaluate assertion script: {}", e)),
    }
}
