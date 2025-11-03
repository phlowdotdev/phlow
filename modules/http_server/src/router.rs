use crate::openapi::{OpenAPIValidator, ValidationResult};
use phlow_sdk::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Router {
    pub openapi_validator: Option<OpenAPIValidator>,
}

#[derive(Debug, Clone)]
pub struct RouteValidationResult {
    pub path_params: HashMap<String, String>,
    pub validation_result: Option<ValidationResult>,
    pub matched_route: Option<String>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            openapi_validator: None,
        }
    }

    /// Validates request and extracts path parameters using OpenAPI spec
    pub fn validate_and_extract(
        &self,
        method: &str,
        path: &str,
        query_params: &HashMap<String, String>,
        body: &Value,
    ) -> RouteValidationResult {
        log::debug!(
            "Router.validate_and_extract: method={} path={} query_params_count={} has_body={}",
            method,
            path,
            query_params.len(),
            !matches!(body, Value::Null | Value::Undefined)
        );
        // If OpenAPI validator is available, use it
        if let Some(validator) = &self.openapi_validator {
            let validation_result = validator.validate_request(method, path, query_params, body);
            log::debug!(
                "OpenAPI validation: is_valid={} matched_route={:?} path_params_keys={:?}",
                validation_result.is_valid,
                validation_result.matched_route,
                validation_result
                    .path_params
                    .keys()
                    .cloned()
                    .collect::<Vec<String>>()
            );

            return RouteValidationResult {
                path_params: validation_result.path_params.clone(),
                validation_result: Some(validation_result.clone()),
                matched_route: validation_result.matched_route.clone(),
            };
        }

        // No OpenAPI spec available - return empty params
        RouteValidationResult {
            path_params: HashMap::new(),
            validation_result: None,
            matched_route: None,
        }
    }
}

impl From<Value> for Router {
    fn from(_value: Value) -> Self {
        // Just return new router - OpenAPI validator will be set separately
        log::debug!("Creating Router from value (OpenAPI validator configured separately)");
        Router::new()
    }
}
