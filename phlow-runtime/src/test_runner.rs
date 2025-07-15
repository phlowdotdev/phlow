use phlow_sdk::valu3::prelude::*;
use phlow_sdk::valu3::value::Value;
use phlow_sdk::prelude::json;
use phlow_engine::phs::{build_engine, Script};
use phlow_engine::{Context, Phlow};
use phlow_sdk::structs::Modules;
use std::sync::Arc;
use std::collections::HashMap;
use crate::loader::Loader;

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

pub async fn run_tests(loader: Loader, test_filter: Option<&str>) -> Result<TestSummary, String> {
    // Get tests from loader.tests, not loader.steps
    let tests = loader.tests.as_ref().ok_or("No tests found in the phlow file")?;
    let steps = &loader.steps;
    
    if !tests.is_array() {
        return Err(format!("Tests must be an array, got: {:?}", tests));
    }

    let test_cases = tests.as_array().unwrap();
    
    // Filter tests if test_filter is provided
    let filtered_tests: Vec<_> = if let Some(filter) = test_filter {
        test_cases.values.iter().enumerate().filter(|(_, test_case)| {
            if let Some(description) = test_case.get("describe") {
                let desc_str = description.as_string();
                return desc_str.contains(filter);
            }
            false
        }).collect()
    } else {
        test_cases.values.iter().enumerate().collect()
    };
    
    let mut results = Vec::new();
    let mut passed = 0;
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
        println!("🧪 Running {} test(s) matching '{}' (out of {} total)...", total, filter, test_cases.len());
    } else {
        println!("🧪 Running {} test(s)...", total);
    }
    println!();

    for (run_index, (original_index, test_case)) in filtered_tests.iter().enumerate() {
        let test_index = run_index + 1;
        
        // Extract test description if available
        let test_description = test_case.get("describe")
            .map(|v| v.as_string());
        
        // Print test header with description
        if let Some(ref desc) = test_description {
            print!("Test {}: {} - ", test_index, desc);
        } else {
            print!("Test {}: ", test_index);
        }
        
        let result = run_single_test(test_case, steps.clone(), *original_index + 1).await;
        
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

async fn run_single_test(test_case: &Value, steps: Value, _test_index: usize) -> Result<String, String> {
    // Extract test inputs
    let main_value = test_case.get("main").cloned().unwrap_or(Value::Null);
    let initial_payload = test_case.get("payload").cloned().unwrap_or(Value::Null);
    
    // Execute the steps with the test inputs
    let result = execute_steps_with_context(steps, main_value, initial_payload).await
        .map_err(|e| format!("Execution error: {}", e))?;
    
    // Check assertions
    if let Some(assert_eq_value) = test_case.get("assert_eq") {
        if result == *assert_eq_value {
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

async fn execute_steps_with_context(steps: Value, main: Value, payload: Value) -> Result<Value, String> {
    // Create a phlow workflow with just the steps
    let workflow = json!({
        "steps": steps
    });
    
    // Create modules (empty for now)
    let modules = Arc::new(Modules::default());
    
    // Create phlow instance
    let phlow = Phlow::try_from_value(&workflow, Some(modules))
        .map_err(|e| format!("Failed to create phlow: {}", e))?;
    
    // Create context with test data
    let mut context = Context::from_main(main);
    
    // Set initial payload if provided
    if !payload.is_null() {
        context = context.add_module_output(payload);
    }
    
    // Execute the workflow
    let result = phlow.execute(&mut context).await
        .map_err(|e| format!("Execution failed: {}", e))?;
    
    Ok(result.unwrap_or(Value::Null))
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
    let context_map: HashMap<String, Value> = [
        ("payload".to_string(), result.clone()),
    ].iter().cloned().collect();

    let assertion_result = script.evaluate(&context_map)
        .map_err(|e| format!("Failed to evaluate assertion: {}", e))?;
    
    // Check if result is boolean true
    match assertion_result {
        Value::Boolean(b) => Ok(b),
        Value::String(s) if s == "true".into() => Ok(true),
        Value::String(s) if s == "false".into() => Ok(false),
        _ => Err(format!("Assertion must return boolean, got: {}", assertion_result)),
    }
}
