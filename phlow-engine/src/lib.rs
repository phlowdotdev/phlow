//! # phlow - A Dynamic Rule-Based Workflow Engine
//!
//! `phlow` is a **flexible and extensible** rule-based workflow engine written in Rust.
//! It allows users to define and execute **dynamic decision trees** and **conditional workflows**
//! using JSON-based configurations and embedded scripting via [`Rhai`](https://rhai.rs).
//!
//! ## Features
//! - **Dynamic workflows** with JSON-defined rules
//! - **Embedded scripting** with Rhai for advanced expressions
//! - **Conditional branching** with assert expressions
//! - **Step-based execution** with context-aware evaluation
//! - **Extensible engine** with pluggable functions
//!
//! ## Example: Decision Tree for Club Membership Approval
//!
//! This example demonstrates a **decision tree** to determine if a person can become a club member.
//!
//! ```rust
//! use phlow_engine::{Phlow, Context};
//! use phs::build_engine;
//! use valu3::prelude::*;
//! use valu3::json;
//! use std::sync::Arc;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let decision_tree = json!({
//!       "steps": [
//!         {
//!           "assert": "{{ main.age >= 18 }}",
//!           "then": {
//!             "steps": [
//!               {
//!                 "assert": "{{ main.income >= 5000.0 }}",
//!                 "then": {
//!                   "return": "Approved"
//!                 },
//!                 "else": {
//!                   "return": "Rejected - Insufficient income"
//!                 }
//!               }
//!             ]
//!           },
//!           "else": {
//!             "return": "Rejected - Underage"
//!           }
//!         }
//!       ]
//!     });
//!
//!     let phlow = Phlow::try_from_value(&decision_tree, None)?;
//!     let mut context = Context::from_main(json!({ "age": 20, "income": 6000.0 }));
//!     let result = phlow.execute(&mut context).await?;
//!
//!     println!("Decision: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`phlow`] - Main structure containing pipelines and execution logic.
//! - [`context`] - Manages execution state and variable storage.
//! - [`pipeline`] - Defines sequential execution of processing steps.
//! - [`step_worker`] - Handles conditional logic and step execution.
//! - [`script`] - Integrates Rhai scripting for dynamic evaluation.
//! - [`engine`] - Configures and extends the scripting engine.
//! - [`condition`] - Evaluates assert expressions for branching.
//! - [`collector`] - Logs execution steps and tracks workflow state.
//!
//! ## Architecture Overview
//!
//! The `phlow` engine processes a **workflow pipeline** composed of steps. Each step can:
//! - Evaluate **conditions** (e.g., comparisons, regex matching)
//! - Execute **scripts** for computations
//! - **Branch** execution based on conditions
//! - Store and reference **previous step outputs**
//!
//! ### Execution Flow
//! 1. The engine receives an input **JSON workflow definition**.
//! 2. The `phlow` instance **parses** and **validates** the workflow.
//! 3. The workflow **executes step-by-step**, evaluating conditions and executing actions.
//! 4. The **final result** is returned to the caller.
//!
//! ## Advanced Usage
//!
//! ### Adding Custom Plugins
//!
//! Users can **extend phlow** by adding custom functions to the execution engine.
//! The phlow engine supports extending functionality through custom Rhai functions
//! and repositories that can be injected into the scripting environment.
//!
//! For detailed examples of extending the engine, see the `phs` module documentation
//! and the `build_engine` function.
//!
//! ### Handling Execution Errors
//!
//! Errors during workflow execution are returned as `Result<T, PhlowError>`:
//!
//! ```rust
//! use phlow_engine::{Phlow, Context};
//! use valu3::prelude::*;
//! use valu3::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let workflow = json!({
//!       "steps": [
//!         {
//!           "return": "Success"
//!         }
//!       ]
//!     });
//!
//!     let phlow = Phlow::try_from_value(&workflow, None)?;
//!     let mut context = Context::from_main(json!({"key": "value"}));
//!
//!     match phlow.execute(&mut context).await {
//!         Ok(result) => println!("Execution succeeded: {:?}", result),
//!         Err(err) => eprintln!("Execution failed: {:?}", err),
//!     }
//! # Ok(())
//! # }
//! ```
//!
//! ## License
//!
//! This project is licensed under the **MIT License**.
pub mod collector;
pub mod condition;
pub mod context;
pub mod debug;
pub mod id;
pub mod phlow;
pub mod pipeline;
pub mod script;
pub mod step_worker;
pub mod transform;
pub use phs;

pub use context::Context;
pub use debug::{
    debug_controller, set_debug_controller, DebugContext, DebugController, DebugReleaseResult,
    DebugSnapshot,
};
pub use phlow::{Phlow, PhlowError};
