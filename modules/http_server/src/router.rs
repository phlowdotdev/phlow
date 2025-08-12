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
        headers: &HashMap<String, String>,
        query_params: &HashMap<String, String>,
        body: &Value,
    ) -> RouteValidationResult {
        // If OpenAPI validator is available, use it
        if let Some(validator) = &self.openapi_validator {
            let validation_result = validator.validate_request(method, path, headers, query_params, body);
            
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
        Router::new()
    }
}
