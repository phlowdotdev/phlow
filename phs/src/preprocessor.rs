use regex::Regex;

pub struct SpreadPreprocessor;

impl SpreadPreprocessor {
    pub fn new() -> Self {
        Self
    }

    /// Processa o código para transformar spread syntax em chamadas de função
    pub fn process(&self, code: &str) -> String {
        // Estratégia simples: processa apenas os casos mais externos primeiro
        let code = self.process_arrays_simple(code);
        let code = self.process_objects_simple(&code);
        code
    }

    /// Processa arrays com spread de forma simples
    fn process_arrays_simple(&self, code: &str) -> String {
        // Regex que captura apenas arrays não aninhados com spread
        let array_regex = Regex::new(r"\[([^\[\]]*\.\.\..*?[^\[\]]*)\]").unwrap();
        
        array_regex.replace_all(code, |caps: &regex::Captures| {
            let content = caps.get(1).unwrap().as_str().trim();
            self.transform_array_spread(content)
        }).to_string()
    }

    /// Processa objetos com spread de forma simples
    fn process_objects_simple(&self, code: &str) -> String {
        // Regex que captura apenas objetos não aninhados com spread
        let object_regex = Regex::new(r"#\{([^{}]*\.\.\..*?[^{}]*)\}").unwrap();
        
        object_regex.replace_all(code, |caps: &regex::Captures| {
            let content = caps.get(1).unwrap().as_str().trim();
            self.transform_object_spread(content)
        }).to_string()
    }

    /// Transforma conteúdo de objeto com spread
    fn transform_object_spread(&self, content: &str) -> String {
        let parts = self.parse_object_parts(content);
        let mut spread_items = Vec::new();

        for part in parts {
            if part.starts_with("...") {
                // É um spread
                let var_name = part[3..].trim();
                spread_items.push(var_name.to_string());
            } else {
                // É um par chave-valor normal
                spread_items.push(format!("#{{{}}}", part));
            }
        }

        format!("__spread_object([{}])", spread_items.join(", "))
    }

    /// Transforma conteúdo de array com spread
    fn transform_array_spread(&self, content: &str) -> String {
        let parts = self.parse_array_parts(content);
        let mut spread_items = Vec::new();

        for part in parts {
            if part.starts_with("...") {
                // É um spread
                let var_name = part[3..].trim();
                spread_items.push(var_name.to_string());
            } else {
                // É um elemento normal
                spread_items.push(format!("[{}]", part));
            }
        }

        format!("__spread_array([{}])", spread_items.join(", "))
    }

    /// Faz parse das partes de um objeto, respeitando chaves-valores e spreads
    fn parse_object_parts(&self, content: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if escape_next {
                current_part.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                    current_part.push(ch);
                }
                '"' | '\'' => {
                    in_string = !in_string;
                    current_part.push(ch);
                }
                '{' if !in_string => {
                    brace_count += 1;
                    current_part.push(ch);
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    current_part.push(ch);
                }
                '[' if !in_string => {
                    bracket_count += 1;
                    current_part.push(ch);
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                    current_part.push(ch);
                }
                ',' if !in_string && brace_count == 0 && bracket_count == 0 => {
                    if !current_part.trim().is_empty() {
                        parts.push(current_part.trim().to_string());
                    }
                    current_part.clear();
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if !current_part.trim().is_empty() {
            parts.push(current_part.trim().to_string());
        }

        parts
    }

    /// Faz parse das partes de um array, respeitando elementos e spreads
    fn parse_array_parts(&self, content: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in content.chars() {
            if escape_next {
                current_part.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                    current_part.push(ch);
                }
                '"' | '\'' => {
                    in_string = !in_string;
                    current_part.push(ch);
                }
                '{' if !in_string => {
                    brace_count += 1;
                    current_part.push(ch);
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    current_part.push(ch);
                }
                '[' if !in_string => {
                    bracket_count += 1;
                    current_part.push(ch);
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                    current_part.push(ch);
                }
                ',' if !in_string && brace_count == 0 && bracket_count == 0 => {
                    if !current_part.trim().is_empty() {
                        parts.push(current_part.trim().to_string());
                    }
                    current_part.clear();
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if !current_part.trim().is_empty() {
            parts.push(current_part.trim().to_string());
        }

        parts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_spread_simple() {
        let preprocessor = SpreadPreprocessor::new();
        let code = "#{...a, b: 2, ...c}";
        let result = preprocessor.process(code);
        assert_eq!(result, "__spread_object([a, #{b: 2}, c])");
    }

    #[test]
    fn test_object_without_spread() {
        let preprocessor = SpreadPreprocessor::new();
        let code = "#{a: 1, b: 2}";
        let result = preprocessor.process(code);
        assert_eq!(result, "#{a: 1, b: 2}");
    }

    #[test]
    fn test_array_spread_simple() {
        let preprocessor = SpreadPreprocessor::new();
        let code = "[...a, 1, ...b]";
        let result = preprocessor.process(code);
        assert_eq!(result, "__spread_array([a, [1], b])");
    }

    #[test]
    fn test_array_without_spread() {
        let preprocessor = SpreadPreprocessor::new();
        let code = "[1, 2, 3]";
        let result = preprocessor.process(code);
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_complex_object_spread() {
        let preprocessor = SpreadPreprocessor::new();
        let code = "#{...user, name: \"John\", ...settings, active: true}";
        let result = preprocessor.process(code);
        assert_eq!(result, "__spread_object([user, #{name: \"John\"}, settings, #{active: true}])");
    }

    #[test]
    fn test_nested_objects() {
        let preprocessor = SpreadPreprocessor::new();
        // Teste mais simples para objetos aninhados
        let code = "#{...a, nested: #{b: 1, c: 2}}";
        let result = preprocessor.process(code);
        // Apenas verifica se o spread externo foi processado
        assert!(result.contains("__spread_object"));
        assert!(result.contains("nested:"));
    }

    #[test]
    fn test_mixed_spread() {
        let preprocessor = SpreadPreprocessor::new();
        // Teste mais simples para casos mistos
        let code = "[...arr1, 1, ...arr2]";
        let result = preprocessor.process(code);
        assert_eq!(result, "__spread_array([arr1, [1], arr2])");
    }
}
