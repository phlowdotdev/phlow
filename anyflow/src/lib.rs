//! # Anyflow
//!
//! Anyflow is a JSON-based rule engine that enables the creation of conditional flows
//! and dynamic rule execution. It supports mathematical, logical, and
//! complex conditional operations, utilizing **Rhai** for script execution.
//!
//! ## 📌 Main Features
//! - Define rules with **conditions, actions, and mathematical expressions**.
//! - Use **Rhai** for advanced rule processing.
//! - Support for **UUIDs**, **regular expressions**, and **advanced validations**.
//! - Process structured rule flows in JSON.
//!
//! ## 🚀 Usage Example
//!
//! ```rust
//! use anyflow::Engine;
//! use std::collections::HashMap;
//!
//! fn main() {
//!     let mut engine = Engine::new();
//!     
//!     let rules = r#"
//!     {
//!         "steps": [
//!             {
//!                 "condition": {
//!                     "left": "params.requested",
//!                     "right": "params.pre-approved",
//!                     "operator": "less_than"
//!                 },
//!                 "then": {
//!                     "return": "params.requested"
//!                 },
//!                 "else": {
//!                     "return": "params.pre-approved"
//!                 }
//!             }
//!         ]
//!     }
//!     "#;
//!     
//!     let mut params = HashMap::new();
//!     params.insert("requested", 5000);
//!     params.insert("pre-approved", 3000);
//!
//!     let result = engine.evaluate(rules, params);
//!     println!("Result: {:?}", result);
//! }
//! ```
//!
//! ## 🔧 Dependencies
//! ```toml
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! rhai = { version = "1.21.0", features = ["serde"] }
//! regex = "1.11.1"
//! uuid = { version = "1.15.1", features = ["v4"] }
//! valu3 = "0.8.2"
//! ```
//!
//! ## 📚 JSON Structure of the Rule Engine
//!
//! A valid JSON configuration example:
//!
//! ```json
//! {
//!   "steps": [
//!     {
//!       "condition": {
//!         "left": "params.requested",
//!         "right": "params.pre-approved",
//!         "operator": "less_than"
//!       },
//!       "then": {
//!         "payload": "params.requested"
//!       },
//!       "else": {
//!         "steps": [
//!           {
//!             "condition": {
//!               "left": "params.score",
//!               "right": 0.5,
//!               "operator": "greater_than"
//!             }
//!           },
//!           {
//!             "id": "approved",
//!             "payload": {
//!               "total": "(params.requested * 0.3) + params.pre-approved"
//!             }
//!           },
//!           {
//!             "condition": {
//!               "left": "steps.approved.total",
//!               "right": "params.requested",
//!               "operator": "greater_than"
//!             },
//!             "then": {
//!               "return": "params.requested"
//!             },
//!             "else": {
//!               "return": "steps.approved.total"
//!             }
//!           }
//!         ]
//!       }
//!     }
//!   ]
//! }
//! ```
//!
//! ## 🛠 How It Works
//! - Each **step** in the flow can contain a **condition**.
//! - If the condition is **true**, the `then` block is executed.
//! - If the condition is **false**, the `else` block is executed.
//! - Supports **mathematical operations** and **chained rules**.
//!
//! ## 📖 Conclusion
//! **AnyFlow** is a lightweight and powerful solution for **rule execution and decision making**,
//! allowing dynamic logic configuration without modifying the source code.
//!
//! 💡 Ideal for **credit systems, automation, and conditional decision-making applications**.

mod anyflow;
mod collector;
mod condition;
mod context;
mod engine;
mod id;
mod pipeline;
mod script;
mod step_worker;
mod transform;
mod variable;

pub use anyflow::Anyflow;
pub use rhai::Engine;

#[macro_export]
macro_rules! anyflow {
    ($value:expr) => {
        let engine = $crate::Engine::new();
        anyflow::try_from_value(&engine, $value, None)
    };
    ($value:expr, $params:expr) => {
        let engine = $crate::Engine::new();
        anyflow::try_from_value(&engine, $value, Some($params))
    };
}
