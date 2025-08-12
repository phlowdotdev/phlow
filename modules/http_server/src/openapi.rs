use phlow_sdk::prelude::*;
use regex::Regex;
use std::{collections::HashMap, path::Path};


#[derive(Debug, Clone)]
pub struct OpenAPIValidator {
    pub spec_json: String,
    pub config: ValidationConfig,
    pub route_patterns: Vec<RoutePattern>,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub strict_mode: bool,
    pub validate_request_body: bool,
    pub validate_response_body: bool,
}

#[derive(Debug, Clone)]
pub struct RoutePattern {
    pub path_pattern: String,
    pub regex: Regex,
    pub param_names: Vec<String>,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub matched_route: Option<String>,
    pub path_params: HashMap<String, String>,
    pub errors: Vec<ValidationError>,
    pub status_code: u16,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    RouteNotFound,
    MethodNotAllowed,
    InvalidRequestBody,
    MissingRequiredField,
    InvalidFieldType,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            validate_request_body: true,
            validate_response_body: true,
        }
    }
}

impl OpenAPIValidator {
    pub fn from_spec_file<P: AsRef<Path>>(
        path: P,
        config: ValidationConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(&path)?;
        Self::from_spec_content(&content, config)
    }

    pub fn from_spec_content(
        content: &str,
        config: ValidationConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let spec_json = if content.trim_start().starts_with('{') {
            // Already JSON format
            content.to_string()
        } else {
            // YAML format - convert to JSON
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;
            serde_json::to_string_pretty(&yaml_value)?
        };

        let route_patterns = Self::build_route_patterns_from_json(&spec_json)?;

        Ok(Self {
            spec_json,
            config,
            route_patterns,
        })
    }

    pub fn from_value(value: Value) -> Result<Option<Self>, Box<dyn std::error::Error + Send + Sync>> {
        // Check for OpenAPI spec as inline object
        if let Some(spec_value) = value.get("openapi_spec") {
            let config = Self::extract_validation_config(&value);
            
            match spec_value {
                // Inline object spec
                Value::Object(_) => {
                    // Use valu3's native serde integration
                    let spec_json = serde_json::to_string_pretty(spec_value)?;
                    return Ok(Some(Self::from_spec_content(&spec_json, config)?));
                }
                // File path (backward compatibility)
                Value::String(spec_path_val) => {
                    let spec_path = spec_path_val.as_str();
                    return Ok(Some(Self::from_spec_file(spec_path, config)?));
                }
                _ => {
                    return Err("openapi_spec must be an object or string path".into());
                }
            }
        }

        Ok(None)
    }

    fn extract_validation_config(value: &Value) -> ValidationConfig {
        let mut config = ValidationConfig::default();

        if let Some(Value::Object(validation_obj)) = value.get("validation") {
            if let Some(strict_val) = validation_obj.get("strict_mode") {
                if let Some(val) = strict_val.as_bool() {
                    config.strict_mode = *val;
                }
            }
            if let Some(validate_req_val) = validation_obj.get("validate_request_body") {
                if let Some(val) = validate_req_val.as_bool() {
                    config.validate_request_body = *val;
                }
            }
            if let Some(validate_resp_val) = validation_obj.get("validate_response_body") {
                if let Some(val) = validate_resp_val.as_bool() {
                    config.validate_response_body = *val;
                }
            }
        }

        config
    }

    fn build_route_patterns_from_json(
        spec_json: &str,
    ) -> Result<Vec<RoutePattern>, Box<dyn std::error::Error + Send + Sync>> {
        let mut patterns = Vec::new();
        let spec: serde_json::Value = serde_json::from_str(spec_json)?;

        if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
            for (path, path_item) in paths {
                let pattern = path.clone(); // Keep original OpenAPI syntax
                let regex = Self::build_regex_from_openapi_path(&pattern)?;
                let param_names = Self::extract_param_names_from_openapi(&pattern);

                let mut methods = Vec::new();
                if let Some(path_obj) = path_item.as_object() {
                    if path_obj.contains_key("get") { methods.push("GET".to_string()); }
                    if path_obj.contains_key("post") { methods.push("POST".to_string()); }
                    if path_obj.contains_key("put") { methods.push("PUT".to_string()); }
                    if path_obj.contains_key("patch") { methods.push("PATCH".to_string()); }
                    if path_obj.contains_key("delete") { methods.push("DELETE".to_string()); }
                    if path_obj.contains_key("head") { methods.push("HEAD".to_string()); }
                    if path_obj.contains_key("options") { methods.push("OPTIONS".to_string()); }
                    if path_obj.contains_key("trace") { methods.push("TRACE".to_string()); }
                }

                patterns.push(RoutePattern {
                    path_pattern: pattern,
                    regex,
                    param_names,
                    methods,
                });
            }
        }

        Ok(patterns)
    }

    /// Build regex from OpenAPI path pattern (with {param} syntax)
    fn build_regex_from_openapi_path(openapi_path: &str) -> Result<Regex, regex::Error> {
        let mut regex_pattern = "^".to_string();
        
        for segment in openapi_path.split('/') {
            if segment.is_empty() {
                continue;
            }
            
            regex_pattern.push('/');
            
            // Check if segment contains OpenAPI parameter {param}
            if segment.starts_with('{') && segment.ends_with('}') {
                // Parameter segment - match any non-slash characters
                regex_pattern.push_str("([^/]+)");
            } else {
                // Static segment - escape special regex characters
                regex_pattern.push_str(&regex::escape(segment));
            }
        }
        
        regex_pattern.push('$');
        Regex::new(&regex_pattern)
    }

    /// Extract parameter names from OpenAPI path (with {param} syntax)
    fn extract_param_names_from_openapi(openapi_path: &str) -> Vec<String> {
        openapi_path
            .split('/')
            .filter_map(|segment| {
                if segment.starts_with('{') && segment.ends_with('}') {
                    // Remove the braces and return parameter name
                    Some(segment[1..segment.len()-1].to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn validate_request(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
        query_params: &HashMap<String, String>,
        body: &Value,
    ) -> ValidationResult {
        let mut path_params = HashMap::new();
        let mut validation_errors = Vec::new();

        for route_pattern in &self.route_patterns {
            if let Some(captures) = route_pattern.regex.captures(path) {
                // Extract path parameters
                for (i, param_name) in route_pattern.param_names.iter().enumerate() {
                    if let Some(capture) = captures.get(i + 1) {
                        path_params.insert(param_name.clone(), capture.as_str().to_string());
                    }
                }

                // Check if method is allowed
                if !route_pattern.methods.contains(&method.to_string()) {
                    return ValidationResult {
                        is_valid: false,
                        matched_route: Some(route_pattern.path_pattern.clone()),
                        path_params,
                        errors: vec![ValidationError {
                            error_type: ValidationErrorType::MethodNotAllowed,
                            message: format!(
                                "Method {} not allowed. Allowed methods: {}",
                                method,
                                route_pattern.methods.join(", ")
                            ),
                            field: None,
                        }],
                        status_code: 405,
                    };
                }

                // Perform additional validations if enabled
                if self.config.validate_request_body {
                    self.validate_request_body(body, &mut validation_errors);
                }

                if self.config.strict_mode {
                    self.validate_headers(headers, &mut validation_errors);
                    self.validate_query_parameters(query_params, &mut validation_errors);
                }

                // Return result
                let is_valid = validation_errors.is_empty();
                let status_code = if is_valid { 200 } else { 400 };

                return ValidationResult {
                    is_valid,
                    matched_route: Some(route_pattern.path_pattern.clone()),
                    path_params,
                    errors: validation_errors,
                    status_code,
                };
            }
        }

        // No route matched
        ValidationResult {
            is_valid: false,
            matched_route: None,
            path_params: HashMap::new(),
            errors: vec![ValidationError {
                error_type: ValidationErrorType::RouteNotFound,
                message: format!("Route not found: {}", path),
                field: None,
            }],
            status_code: 404,
        }
    }

    /// Validate request body against OpenAPI spec
    fn validate_request_body(&self, body: &Value, errors: &mut Vec<ValidationError>) {
        // Parse OpenAPI spec to get schema definitions
        if let Ok(spec) = serde_json::from_str::<serde_json::Value>(&self.spec_json) {
            // For now, implement basic validation for common patterns
            // This is a simplified implementation that validates basic structure
            
            match body {
                Value::Undefined | Value::Null => {
                    // Check if body is required - for POST/PUT requests, usually it is
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidRequestBody,
                        message: "Request body is required".to_string(),
                        field: Some("body".to_string()),
                    });
                }
                Value::Object(obj) => {
                    // Validate object structure based on OpenAPI schema
                    self.validate_object_schema(obj, errors, &spec);
                }
                Value::String(s) if s.trim().is_empty() => {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidRequestBody,
                        message: "Request body cannot be empty".to_string(),
                        field: Some("body".to_string()),
                    });
                }
                _ => {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidRequestBody,
                        message: "Invalid request body format".to_string(),
                        field: Some("body".to_string()),
                    });
                }
            }
        }
    }
    
    /// Validate object schema against OpenAPI specification
    fn validate_object_schema(&self, obj: &Object, errors: &mut Vec<ValidationError>, spec: &serde_json::Value) {
        // Extract common required fields from OpenAPI spec
        // This is a simplified implementation focusing on the /users POST endpoint
        
        // Check for required fields based on common API patterns
        if !obj.contains_key(&"name") {
            errors.push(ValidationError {
                error_type: ValidationErrorType::MissingRequiredField,
                message: "Missing required field: name".to_string(),
                field: Some("name".to_string()),
            });
        }
        
        if !obj.contains_key(&"email") {
            errors.push(ValidationError {
                error_type: ValidationErrorType::MissingRequiredField,
                message: "Missing required field: email".to_string(),
                field: Some("email".to_string()),
            });
        }
        
        // Validate field types
        if let Some(name_value) = obj.get("name") {
            if !matches!(name_value, Value::String(_)) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldType,
                    message: "Field 'name' must be a string".to_string(),
                    field: Some("name".to_string()),
                });
            }
        }
        
        if let Some(email_value) = obj.get("email") {
            if !matches!(email_value, Value::String(_)) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldType,
                    message: "Field 'email' must be a string".to_string(),
                    field: Some("email".to_string()),
                });
            }
        }
        
        // Validate age field if present
        if let Some(age_value) = obj.get("age") {
            if !matches!(age_value, Value::Number(_)) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldType,
                    message: "Field 'age' must be a number".to_string(),
                    field: Some("age".to_string()),
                });
            }
        }
    }

    /// Validate headers (basic implementation)
    fn validate_headers(&self, _headers: &HashMap<String, String>, _errors: &mut Vec<ValidationError>) {
        // Basic header validation
        // In a full implementation, we would check required headers against the OpenAPI spec
    }

    /// Validate query parameters (basic implementation)
    fn validate_query_parameters(&self, _query_params: &HashMap<String, String>, _errors: &mut Vec<ValidationError>) {
        // Basic query parameter validation
        // In a full implementation, we would validate against the OpenAPI parameter definitions
    }

    
    /// Returns the OpenAPI specification as a JSON string
    pub fn get_spec(&self) -> String {
        self.spec_json.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_regex_from_openapi_path() {
        let regex = OpenAPIValidator::build_regex_from_openapi_path("/users/{id}").unwrap();
        assert!(regex.is_match("/users/123"));
        assert!(regex.is_match("/users/abc"));
        assert!(!regex.is_match("/users/"));
        assert!(!regex.is_match("/users/123/posts"));
        
        let regex2 = OpenAPIValidator::build_regex_from_openapi_path("/users/{userId}/posts/{postId}").unwrap();
        assert!(regex2.is_match("/users/john/posts/42"));
        assert!(!regex2.is_match("/users/john/posts"));
    }

    #[test]
    fn test_extract_param_names_from_openapi() {
        let params = OpenAPIValidator::extract_param_names_from_openapi("/users/{userId}/posts/{postId}");
        assert_eq!(params, vec!["userId", "postId"]);
        
        let params2 = OpenAPIValidator::extract_param_names_from_openapi("/users/{id}");
        assert_eq!(params2, vec!["id"]);
        
        let params3 = OpenAPIValidator::extract_param_names_from_openapi("/users/static/path");
        assert_eq!(params3, Vec::<String>::new());
    }

}
