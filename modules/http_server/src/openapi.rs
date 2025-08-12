use phlow_sdk::prelude::*;
use regex::Regex;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone)]
pub struct OpenAPIValidator {
    pub spec_json: String,
    pub spec: Object,
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
    InvalidFieldValue,
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

    pub fn json_to_value(
        target: &str,
    ) -> Result<(Value, String), Box<dyn std::error::Error + Send + Sync>> {
        let target: String = target.to_string().replace("\\", "\\\\");

        if target.trim_start().starts_with('{') {
            let value = Value::json_to_value(&target)
                .map_err(|e| format!("Failed to parse OpenAPI spec JSON: {:?}", e))?;
            Ok((value, target))
        } else {
            let value: Value = serde_yaml::from_str(&target)?;
            let string = value.to_json(JsonMode::Inline);
            Ok((value, string))
        }
    }

    pub fn from_spec_content(
        content: &str,
        config: ValidationConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("Loading OpenAPI spec from content: {}", content);
        let (spec, spec_json) = Self::json_to_value(content)?;

        let route_patterns = Self::build_route_patterns_from_json(&spec_json)?;
        let spec = match spec.as_object() {
            Some(obj) => obj.clone(),
            None => return Err("Invalid OpenAPI spec format".into()),
        };

        Ok(Self {
            spec_json,
            spec,
            config,
            route_patterns,
        })
    }

    pub fn from_value(
        value: Value,
    ) -> Result<Option<Self>, Box<dyn std::error::Error + Send + Sync>> {
        // Check for OpenAPI spec as inline object
        if let Some(spec_value) = value.get("openapi_spec") {
            let config = Self::extract_validation_config(&value);

            match spec_value {
                // Inline object spec
                Value::Object(_) => {
                    // Use value's native serde integration
                    let spec_json = spec_value.to_json(JsonMode::Inline);
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
        let (spec, _) = Self::json_to_value(spec_json)?;

        if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
            for (path, path_item) in paths.iter() {
                let pattern = path.to_string(); // Keep original OpenAPI syntax
                let regex = Self::build_regex_from_openapi_path(&pattern)?;
                let param_names = Self::extract_param_names_from_openapi(&pattern);

                let mut methods = Vec::new();
                if let Some(path_obj) = path_item.as_object() {
                    if path_obj.contains_key(&"get") {
                        methods.push("GET".to_string());
                    }
                    if path_obj.contains_key(&"post") {
                        methods.push("POST".to_string());
                    }
                    if path_obj.contains_key(&"put") {
                        methods.push("PUT".to_string());
                    }
                    if path_obj.contains_key(&"patch") {
                        methods.push("PATCH".to_string());
                    }
                    if path_obj.contains_key(&"delete") {
                        methods.push("DELETE".to_string());
                    }
                    if path_obj.contains_key(&"head") {
                        methods.push("HEAD".to_string());
                    }
                    if path_obj.contains_key(&"options") {
                        methods.push("OPTIONS".to_string());
                    }
                    if path_obj.contains_key(&"trace") {
                        methods.push("TRACE".to_string());
                    }
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
                    Some(segment[1..segment.len() - 1].to_string())
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
                    // Only validate request body for methods that typically have bodies
                    let method_has_body =
                        matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH");
                    if method_has_body {
                        // Pass the actual matched route pattern for dynamic validation
                        self.validate_request_body_with_route(
                            body,
                            &mut validation_errors,
                            method.to_lowercase().as_str(),
                            &route_pattern.path_pattern,
                        );
                    }
                }

                if self.config.strict_mode {
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

    /// Check if request body is required in OpenAPI spec
    fn is_request_body_required(&self, method: &str, route_pattern: &str) -> bool {
        self.spec
            .get("paths")
            .and_then(|paths| paths.get(route_pattern))
            .and_then(|path_item| path_item.get(method.to_lowercase()))
            .and_then(|operation| operation.get("requestBody"))
            .and_then(|req_body| req_body.get("required"))
            .and_then(|required| required.as_bool())
            .map(|b| *b)
            .unwrap_or(false) // Default: body is not required
    }

    /// Validate request body against OpenAPI spec with dynamic route
    fn validate_request_body_with_route(
        &self,
        body: &Value,
        errors: &mut Vec<ValidationError>,
        method: &str,
        route_pattern: &str,
    ) {
        match body {
            Value::Undefined | Value::Null => {
                // Check if body is actually required in the OpenAPI spec
                if self.is_request_body_required(method, route_pattern) {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidRequestBody,
                        message: "Request body is required".to_string(),
                        field: Some("body".to_string()),
                    });
                }
                // If body is not required, null/undefined is valid
            }
            Value::Object(obj) => {
                // Body is present, validate its structure
                self.validate_object_schema_with_route(obj, errors, method, route_pattern);
            }
            Value::String(s) if s.trim().is_empty() => {
                // Empty string body - check if required
                if self.is_request_body_required(method, route_pattern) {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidRequestBody,
                        message: "Request body cannot be empty".to_string(),
                        field: Some("body".to_string()),
                    });
                }
            }
            _ => {
                // Body has some other type - usually invalid for JSON APIs
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidRequestBody,
                    message: "Invalid request body format".to_string(),
                    field: Some("body".to_string()),
                });
            }
        }
    }

    /// Validate object schema against OpenAPI specification with dynamic route
    fn validate_object_schema_with_route(
        &self,
        obj: &Object,
        errors: &mut Vec<ValidationError>,
        method: &str,
        route_pattern: &str,
    ) {
        // Try to extract actual schema from OpenAPI spec for more accurate validation using the actual HTTP method and route
        let schema = self.extract_request_body_schema(route_pattern, method);

        if let Some(properties) = schema.as_ref().and_then(|s| s.get("properties")) {
            // Validate based on actual OpenAPI schema using the correct HTTP method and route
            self.validate_properties_with_schema(obj, properties, errors, schema.as_ref());
        } else {
            // Fallback to basic validation with method awareness
            self.validate_basic_user_schema_with_method(obj, errors, method);
        }
    }

    /// Extract request body schema from OpenAPI spec for a specific path/method
    fn extract_request_body_schema(&self, path: &str, method: &str) -> Option<Value> {
        // Extract each part safely with better error handling
        let paths = self.spec.get("paths")?;
        let path_item = paths.get(path)?;
        let operation = path_item.get(method)?;
        
        // If operation not found, try lowercase method name (per OpenAPI spec)
        let operation = if operation.is_null() || operation.is_undefined() {
            path_item.get(method.to_lowercase())?
        } else {
            operation
        };
        
        // Try to get schema
        let request_body = operation.get("requestBody")?;
        let content = request_body.get("content")?;
        let json_content = content.get("application/json")?;
        let schema = json_content.get("schema")?;
        
        // Resolve schema reference if it's a $ref
        let resolved_schema = self.resolve_schema_reference(schema);
        
        Some(resolved_schema)
    }
    
    /// Resolve schema reference ($ref) to actual schema definition
    fn resolve_schema_reference(&self, schema: &Value) -> Value {
        // Check if this is a schema reference
        if let Some(ref_str) = schema.get("$ref").and_then(|r| {
            let string = r.to_string();
            if string.is_empty() { None } else { Some(string) }
        }) {
            // Parse the reference path (e.g., "#/components/schemas/NewUser")
            if ref_str.starts_with("#/") {
                let path_parts: Vec<&str> = ref_str[2..].split('/').collect();
                
                // Navigate through the spec to find the referenced schema
                let mut current = &Value::Object(self.spec.clone());
                for part in path_parts {
                    if let Some(next) = current.get(part) {
                        current = next;
                    } else {
                        // Reference not found, return original schema
                        log::warn!("Schema reference '{}' not found", ref_str);
                        return schema.clone();
                    }
                }
                
                // Return the resolved schema
                current.clone()
            } else {
                // External reference - not supported, return original
                log::warn!("External schema reference '{}' not supported", ref_str);
                schema.clone()
            }
        } else {
            // Not a reference, return as-is
            schema.clone()
        }
    }

    /// Validate object properties against OpenAPI schema with patterns
    fn validate_properties_with_schema(
        &self,
        obj: &Object,
        properties: &Value,
        errors: &mut Vec<ValidationError>,
        parent_schema: Option<&Value>,
    ) {
        if let Some(props) = properties.as_object() {
            // Extract required fields array from the parent schema that was passed
            let required_fields: Vec<String> = parent_schema
                .and_then(|s| s.get("required"))
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.into_iter()
                        .filter_map(|v| {
                            let string = v.to_string();

                            if string.is_empty() {
                                None
                            } else {
                                Some(string)
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Check if additionalProperties are allowed
            let allow_additional_properties = parent_schema
                .and_then(|s| s.get("additionalProperties"))
                .map(|ap| !matches!(ap, Value::Boolean(false)))
                .unwrap_or(true); // Default to true if not specified

            // Validate each property in the schema
            for (prop_name, prop_schema) in props.iter() {
                let prop_name = prop_name.to_string();
                if let Some(field_value) = obj.get(prop_name.clone()) {
                    // Field is present, validate its value
                    match prop_schema.get("type").and_then(|t| {
                        let string = t.to_string();
                        if string.is_empty() {
                            None
                        } else {
                            Some(string)
                        }
                    }) {
                        Some(ref val) if val == "string" => {
                            self.validate_string_field(
                                &prop_name,
                                field_value,
                                prop_schema,
                                errors,
                            );
                        }
                        Some(ref val) if val == "integer" || val == "number" => {
                            self.validate_numeric_field(
                                &prop_name,
                                field_value,
                                prop_schema,
                                errors,
                            );
                        }
                        Some(ref val) if val == "boolean" => {
                            self.validate_boolean_field(
                                &prop_name,
                                field_value,
                                prop_schema,
                                errors,
                            );
                        }
                        Some(ref val) if val == "array" => {
                            self.validate_array_field(
                                &prop_name,
                                field_value,
                                prop_schema,
                                errors,
                            );
                        }
                        _ => {
                            // Handle other types or generic validation
                        }
                    }
                } else if required_fields.contains(&prop_name.to_string()) {
                    // Field is missing and required
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::MissingRequiredField,
                        message: format!("Missing required field: {}", prop_name),
                        field: Some(prop_name.to_string()),
                    });
                }
                // If field is not present and not required, it's valid (optional field)
            }

            // Check for additional properties if not allowed
            if !allow_additional_properties {
                for field_name in obj.keys() {
                    let field_name_str = field_name.to_string();
                    if !props.contains_key(&field_name_str) {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::InvalidFieldValue,
                            message: format!(
                                "Additional property '{}' is not allowed",
                                field_name_str
                            ),
                            field: Some(field_name_str),
                        });
                    }
                }
            }
        }
    }

    /// Validate string field with pattern/format constraints
    fn validate_string_field(
        &self,
        field_name: &str,
        value: &Value,
        schema: &Value,
        errors: &mut Vec<ValidationError>,
    ) {
        let Value::String(string_val) = value else {
            errors.push(ValidationError {
                error_type: ValidationErrorType::InvalidFieldType,
                message: format!("Field '{}' must be a string", field_name),
                field: Some(field_name.to_string()),
            });
            return;
        };

        let str_val = string_val.as_str();

        // Check minLength
        if let Some(min_len) = schema.get("minLength").and_then(|v| {
            v.as_number()
                .unwrap_or(Value::from(0).as_number().unwrap())
                .to_i64()
        }) {
            if str_val.len() < min_len as usize {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldValue,
                    message: format!(
                        "Field '{}' must be at least {} characters long",
                        field_name, min_len
                    ),
                    field: Some(field_name.to_string()),
                });
            }
        }

        // Check maxLength
        if let Some(max_len) = schema.get("maxLength").and_then(|v| {
            v.as_number()
                .unwrap_or(Value::from(0).as_number().unwrap())
                .to_i64()
        }) {
            if str_val.len() > max_len as usize {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldValue,
                    message: format!(
                        "Field '{}' must be at most {} characters long",
                        field_name, max_len
                    ),
                    field: Some(field_name.to_string()),
                });
            }
        }

        // Check pattern (regex) - with better error handling to prevent panics
        if let Some(pattern) = schema.get("pattern").and_then(|v| {
            // Extract the string value properly without quotes
            let string = match v {
                Value::String(s) => s.as_str().to_string(),
                _ => v.to_string(),
            };

            if string.is_empty() {
                None
            } else {
                Some(string)
            }
        }) {
            match Regex::new(&pattern) {
                Ok(regex) => {
                    log::debug!("Validating field '{}' with value '{}' against pattern '{}'", field_name, str_val, pattern);
                    if !regex.is_match(str_val) {
                        log::debug!("Pattern match failed for '{}' with pattern '{}'", str_val, pattern);
                        let message = if field_name == "email" {
                            format!("Field '{}' must be a valid email address", field_name)
                        } else {
                            format!("Field '{}' format is invalid", field_name)
                        };
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::InvalidFieldValue,
                            message,
                            field: Some(field_name.to_string()),
                        });
                    } else {
                        log::debug!("Pattern match successful for '{}' with pattern '{}'", str_val, pattern);
                    }
                }
                Err(regex_err) => {
                    // If regex is invalid, log the error and treat as validation failure
                    log::warn!("Invalid regex pattern '{}' for field '{}': {}", pattern, field_name, regex_err);
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidFieldValue,
                        message: format!("Field '{}' has invalid validation pattern", field_name),
                        field: Some(field_name.to_string()),
                    });
                }
            }
        }

        // Check format (built-in formats like 'email')
        if let Some(format) = schema.get("format").and_then(|v| {
            let string = v.to_string();

            if string.is_empty() {
                None
            } else {
                Some(string)
            }
        }) {
            if format == "email" && !self.is_valid_email_format(str_val) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldValue,
                    message: format!("Field '{}' must be a valid email address", field_name),
                    field: Some(field_name.to_string()),
                });
            }
        }
    }

    /// Validate numeric field with min/max constraints
    fn validate_numeric_field(
        &self,
        field_name: &str,
        value: &Value,
        schema: &Value,
        errors: &mut Vec<ValidationError>,
    ) {
        // First check if it's actually a number type
        let Value::Number(number) = value else {
            errors.push(ValidationError {
                error_type: ValidationErrorType::InvalidFieldType,
                message: format!("Field '{}' must be a number", field_name),
                field: Some(field_name.to_string()),
            });
            return;
        };

        // Check if schema specifies integer type and value is not an integer
        if let Some(type_str) = schema.get("type").and_then(|t| {
            match t {
                Value::String(s) => Some(s.as_str().to_string()),
                _ => Some(t.to_string()),
            }
        }) {
            if type_str == "integer" {
                // For integer type, check if the number is actually an integer
                let num_val = match number.to_f64() {
                    Some(val) => val,
                    None => {
                        log::warn!("Failed to convert number value for field '{}' to f64", field_name);
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::InvalidFieldValue,
                            message: format!("Field '{}' has invalid numeric value", field_name),
                            field: Some(field_name.to_string()),
                        });
                        return;
                    }
                };
                
                // Check if the value is actually an integer (no decimal part)
                if num_val.fract() != 0.0 {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidFieldType,
                        message: format!("Field '{}' must be an integer (no decimal places)", field_name),
                        field: Some(field_name.to_string()),
                    });
                    return;
                }
            }
        }

        // Try to safely extract the numeric value
        let num_val = match number.to_f64() {
            Some(val) => val,
            None => {
                // If conversion fails, log and return error
                log::warn!("Failed to convert number value for field '{}' to f64", field_name);
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldValue,
                    message: format!("Field '{}' has invalid numeric value", field_name),
                    field: Some(field_name.to_string()),
                });
                return;
            }
        };

        // Check minimum - with safe extraction
        if let Some(min_value) = schema.get("minimum") {
            if let Some(min_number) = min_value.as_number() {
                if let Some(min) = min_number.to_f64() {
                    if num_val < min {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::InvalidFieldValue,
                            message: format!("Field '{}' must be at least {}", field_name, min),
                            field: Some(field_name.to_string()),
                        });
                    }
                }
            }
        }

        // Check maximum - with safe extraction
        if let Some(max_value) = schema.get("maximum") {
            if let Some(max_number) = max_value.as_number() {
                if let Some(max) = max_number.to_f64() {
                    if num_val > max {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::InvalidFieldValue,
                            message: format!("Field '{}' must be at most {}", field_name, max),
                            field: Some(field_name.to_string()),
                        });
                    }
                }
            }
        }
    }

    /// Validate boolean field
    fn validate_boolean_field(
        &self,
        field_name: &str,
        value: &Value,
        _schema: &Value, // For future extensions
        errors: &mut Vec<ValidationError>,
    ) {
        match value {
            Value::Boolean(_) => {
                // Field is valid boolean
            }
            _ => {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldType,
                    message: format!("Field '{}' must be a boolean", field_name),
                    field: Some(field_name.to_string()),
                });
            }
        }
    }

    /// Validate array field with item type validation
    fn validate_array_field(
        &self,
        field_name: &str,
        value: &Value,
        schema: &Value,
        errors: &mut Vec<ValidationError>,
    ) {
        let Value::Array(arr) = value else {
            errors.push(ValidationError {
                error_type: ValidationErrorType::InvalidFieldType,
                message: format!("Field '{}' must be an array", field_name),
                field: Some(field_name.to_string()),
            });
            return;
        };

        // Validate type of array items if specified
        if let Some(items_schema) = schema.get("items") {
            if let Some(item_type) = items_schema.get("type").and_then(|t| {
                let string = t.to_string();
                if string.is_empty() {
                    None
                } else {
                    Some(string)
                }
            }) {
                for (index, item) in arr.values.iter().enumerate() {
                    match item_type.as_str() {
                        "string" => {
                            if !matches!(item, Value::String(_)) {
                                errors.push(ValidationError {
                                    error_type: ValidationErrorType::InvalidFieldType,
                                    message: format!("Array item at index {} must be a string", index),
                                    field: Some(format!("{}[{}]", field_name, index)),
                                });
                            }
                        }
                        "number" | "integer" => {
                            if !matches!(item, Value::Number(_)) {
                                errors.push(ValidationError {
                                    error_type: ValidationErrorType::InvalidFieldType,
                                    message: format!("Array item at index {} must be a number", index),
                                    field: Some(format!("{}[{}]", field_name, index)),
                                });
                            }
                        }
                        "boolean" => {
                            if !matches!(item, Value::Boolean(_)) {
                                errors.push(ValidationError {
                                    error_type: ValidationErrorType::InvalidFieldType,
                                    message: format!("Array item at index {} must be a boolean", index),
                                    field: Some(format!("{}[{}]", field_name, index)),
                                });
                            }
                        }
                        _ => {
                            // Handle other item types if needed
                        }
                    }
                }
            }
        }

        // Check minItems
        if let Some(min_items) = schema.get("minItems").and_then(|v| {
            v.as_number()
                .unwrap_or(Value::from(0).as_number().unwrap())
                .to_i64()
        }) {
            if arr.len() < min_items as usize {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldValue,
                    message: format!(
                        "Field '{}' must have at least {} items",
                        field_name, min_items
                    ),
                    field: Some(field_name.to_string()),
                });
            }
        }

        // Check maxItems
        if let Some(max_items) = schema.get("maxItems").and_then(|v| {
            v.as_number()
                .unwrap_or(Value::from(0).as_number().unwrap())
                .to_i64()
        }) {
            if arr.len() > max_items as usize {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidFieldValue,
                    message: format!(
                        "Field '{}' must have at most {} items",
                        field_name, max_items
                    ),
                    field: Some(field_name.to_string()),
                });
            }
        }
    }

    /// Basic email format validation (fallback)
    fn is_valid_email_format(&self, email: &str) -> bool {
        // Basic validation checks first
        if email.is_empty() || !email.contains('@') {
            return false;
        }
        
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false; // Should have exactly one @
        }
        
        let username = parts[0];
        let domain = parts[1];
        
        // Username checks
        if username.is_empty() || username.starts_with('.') || username.ends_with('.') || username.contains("..") {
            return false;
        }
        
        // Domain checks
        if domain.is_empty() || domain.starts_with('.') || domain.ends_with('.') 
            || domain.starts_with('-') || domain.ends_with('-') || domain.contains("..") 
            || !domain.contains('.') {
            return false;
        }
        
        // Check if domain has a valid TLD (at least 2 characters after last dot)
        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.len() < 2 {
            return false; // Must have at least domain.tld
        }
        
        let tld = domain_parts.last().unwrap();
        if tld.len() < 2 || tld.is_empty() {
            return false;
        }
        
        // More permissive regex that accepts common email formats
        let email_regex = match Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$"
        ) {
            Ok(regex) => regex,
            Err(_) => {
                // If regex compilation fails, use basic validation
                return username.len() > 0 && domain.len() > 3 && domain.contains('.') && !domain.ends_with('.');
            }
        };
        
        email_regex.is_match(email)
    }

    /// Fallback basic validation with HTTP method awareness
    fn validate_basic_user_schema_with_method(
        &self,
        obj: &Object,
        errors: &mut Vec<ValidationError>,
        method: &str,
    ) {
        // For POST requests, require name and email fields (strict validation)
        // For PUT/PATCH requests, allow partial updates (no required fields)
        let is_create_operation = method.to_uppercase() == "POST";

        if is_create_operation {
            // POST requires name and email
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
        }
        // For PUT/PATCH, we don't require any fields (allow partial updates)

        // Validate field types for any fields that are present
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

    /// Validate query parameters (basic implementation)
    fn validate_query_parameters(
        &self,
        _query_params: &HashMap<String, String>,
        _errors: &mut Vec<ValidationError>,
    ) {
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

        let regex2 =
            OpenAPIValidator::build_regex_from_openapi_path("/users/{userId}/posts/{postId}")
                .unwrap();
        assert!(regex2.is_match("/users/john/posts/42"));
        assert!(!regex2.is_match("/users/john/posts"));
    }

    #[test]
    fn test_extract_param_names_from_openapi() {
        let params =
            OpenAPIValidator::extract_param_names_from_openapi("/users/{userId}/posts/{postId}");
        assert_eq!(params, vec!["userId", "postId"]);

        let params2 = OpenAPIValidator::extract_param_names_from_openapi("/users/{id}");
        assert_eq!(params2, vec!["id"]);

        let params3 = OpenAPIValidator::extract_param_names_from_openapi("/users/static/path");
        assert_eq!(params3, Vec::<String>::new());
    }

    #[test]
    fn test_method_aware_validation() {
        let mut obj_map = HashMap::new();
        obj_map.insert("age".to_string(), Value::Number(30.into()));
        let obj = Object::from(obj_map);
        let spec_map: HashMap<String, Value> = HashMap::new();

        let validator = OpenAPIValidator {
            spec_json: String::new(),
            spec: Value::from(spec_map).as_object().unwrap().clone(),
            config: ValidationConfig::default(),
            route_patterns: Vec::new(),
        };

        let mut post_errors = Vec::new();
        let mut put_errors = Vec::new();

        // Test POST - should require name and email
        validator.validate_basic_user_schema_with_method(&obj, &mut post_errors, "POST");

        // Test PUT - should allow partial update (no required fields)
        validator.validate_basic_user_schema_with_method(&obj, &mut put_errors, "PUT");

        // POST should have 2 errors (missing name and email)
        assert_eq!(post_errors.len(), 2);
        assert!(post_errors
            .iter()
            .any(|e| e.message.contains("Missing required field: name")));
        assert!(post_errors
            .iter()
            .any(|e| e.message.contains("Missing required field: email")));

        // PUT should have no errors (allows partial updates)
        assert_eq!(put_errors.len(), 0);
    }
}
