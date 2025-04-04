#[derive(Debug, Clone, PartialEq)]
pub enum AuthorizationSpanMode {
    None,
    Hidden,
    Prefix,
    Suffix,
    All,
}

impl AuthorizationSpanMode {
    pub fn from_str(mode: &str) -> Self {
        match mode {
            "none" => AuthorizationSpanMode::None,
            "hidden" => AuthorizationSpanMode::Hidden,
            "prefix" => AuthorizationSpanMode::Prefix,
            "suffix" => AuthorizationSpanMode::Suffix,
            "all" => AuthorizationSpanMode::All,
            _ => AuthorizationSpanMode::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub authorization_span_mode: AuthorizationSpanMode,
}

impl Settings {
    pub fn load() -> Self {
        let authorization_span_mode = match std::env::var("PHLOW_AUTHORIZATION_SPAN_MODE") {
            Ok(mode) => AuthorizationSpanMode::from_str(&mode),
            Err(_) => AuthorizationSpanMode::Prefix,
        };

        Settings {
            authorization_span_mode,
        }
    }
}
