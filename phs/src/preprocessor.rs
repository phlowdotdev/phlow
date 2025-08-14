use regex::Regex;

pub struct SpreadPreprocessor;

impl SpreadPreprocessor {
    pub fn new() -> Self {
        Self
    }

    /// Processa o código para transformar spread syntax em chamadas de função
    pub fn process(&self, code: &str) -> String {
        // Primeiro, converte objetos simples para a sintaxe Rhai com #
        let code = self.process_object_literals(&code);
        // Depois processa arrays com spread
        let code = self.process_arrays_simple(&code);
        // Por último, processa objetos com spread
        let code = self.process_objects_simple(&code);
        code
    }

    /// Converte objetos literais {key: value} para #{key: value} para compatibilidade com Rhai
    fn process_object_literals(&self, code: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = code.chars().collect();

        while i < chars.len() {
            if chars[i] == '{' {
                // Verifica se é um objeto literal ou outro tipo de bloco
                if self.is_object_literal(&chars, i) {
                    // É um objeto literal, adiciona o # na frente
                    result.push('#');
                    result.push('{');
                    i += 1;
                } else {
                    // Não é um objeto literal, mantém como está
                    result.push(chars[i]);
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// Determina se uma abertura de chaves '{' representa um objeto literal
    fn is_object_literal(&self, chars: &[char], start_pos: usize) -> bool {
        if start_pos >= chars.len() || chars[start_pos] != '{' {
            return false;
        }

        // Encontra o conteúdo entre as chaves
        if let Some((_end_pos, content)) =
            self.find_matching_brace_from(chars, start_pos + 1, '{', '}')
        {
            return self.analyze_brace_content(&content);
        }

        false
    }

    /// Analisa o conteúdo entre chaves para determinar se é um objeto literal
    fn analyze_brace_content(&self, content: &str) -> bool {
        let content = content.trim();

        // Objeto vazio é um objeto literal
        if content.is_empty() {
            return true;
        }

        // Se tem declarações de let/const/var, é um bloco de código
        if content.contains("let ") || content.contains("const ") || content.contains("var ") {
            return false;
        }

        // Verifica se contém spread syntax (definitivamente um objeto)
        // MAS apenas se não contém outras estruturas de código
        if content.contains("...") {
            // Verifica se não é um bloco complexo com múltiplas estruturas
            let lines = content.lines().collect::<Vec<_>>();
            let mut has_object_structure = false;
            let mut has_code_structure = false;

            for line in lines {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Padrões de código
                if line.starts_with("if ")
                    || line.starts_with("for ")
                    || line.starts_with("while ")
                    || line.starts_with("return ")
                    || line.starts_with("let ")
                    || line.starts_with("const ")
                    || line.starts_with("var ")
                    || line.contains("();")
                {
                    has_code_structure = true;
                }

                // Padrões de objeto
                if line.contains(":")
                    && !line.contains("();")
                    && !line.starts_with("if")
                    && !line.starts_with("for")
                    && !line.starts_with("while")
                {
                    has_object_structure = true;
                }
            }

            // Se tem apenas estrutura de objeto, é um objeto
            return has_object_structure && !has_code_structure;
        }

        // Verifica padrões que indicam que NÃO é um objeto literal
        let non_object_patterns = [
            // Estruturas de controle
            r"^\s*if\s*\(",
            r"^\s*for\s*\(",
            r"^\s*while\s*\(",
            r"^\s*switch\s*\(",
            r"^\s*try\s*{",
            r"^\s*catch\s*\(",
            // Funções
            r"^\s*function\s+",
            r"^\s*fn\s+",
            // Statements que indicam bloco de código
            r"^\s*return\s+",
            r"^\s*break\s*;",
            r"^\s*continue\s*;",
            // Múltiplas linhas com statements
            r"\n\s*\w+\s*\(",
        ];

        for pattern in &non_object_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(content) {
                    return false;
                }
            }
        }

        // Verifica se tem padrão de objeto: chave: valor ou "chave": valor
        let object_patterns = [
            // Chave simples seguida de dois pontos (no início da linha)
            r"^\s*\w+\s*:\s*",
            // Chave entre aspas seguida de dois pontos
            r#"^\s*["']\w+["']\s*:\s*"#,
            // Spread syntax (no início da linha)
            r"^\s*\.\.\.\w+",
        ];

        for pattern in &object_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(content) {
                    return true;
                }
            }
        }

        // Se chegou até aqui e tem dois pontos sem ponto e vírgula, pode ser objeto
        // mas precisa ser mais cuidadoso
        if content.contains(':') && !content.contains(';') {
            // Verifica se não é um bloco complexo
            let lines = content.lines().count();
            if lines > 3 {
                // Muitas linhas, provavelmente é código
                return false;
            }
            return true;
        }

        false
    }

    /// Processa arrays com spread de forma simples
    fn process_arrays_simple(&self, code: &str) -> String {
        // Regex que captura apenas arrays não aninhados com spread
        let array_regex = Regex::new(r"\[([^\[\]]*\.\.\..*?[^\[\]]*)\]").unwrap();

        array_regex
            .replace_all(code, |caps: &regex::Captures| {
                let content = caps.get(1).unwrap().as_str().trim();
                self.transform_array_spread(content)
            })
            .to_string()
    }

    /// Processa objetos com spread de forma simples
    fn process_objects_simple(&self, code: &str) -> String {
        // Usa função customizada para lidar com objetos aninhados
        self.find_and_replace_objects_with_spread(code)
    }

    /// Encontra e substitui objetos que contêm spread, lidando com aninhamento
    fn find_and_replace_objects_with_spread(&self, code: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = code.chars().collect();

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '#' && chars[i + 1] == '{' {
                // Encontrou início de objeto Rhai (já processado)
                if let Some((end_pos, content)) =
                    self.find_matching_brace_from(&chars, i + 2, '{', '}')
                {
                    if content.contains("...") {
                        // Tem spread, processa
                        let transformed = self.transform_object_spread(&content);
                        result.push_str(&transformed);
                    } else {
                        // Não tem spread, mantém original
                        result.push_str(&format!("#{{{}}}", content));
                    }
                    i = end_pos + 1;
                } else {
                    // Não encontrou fechamento, mantém caracteres originais
                    result.push(chars[i]);
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// Encontra chave de fechamento correspondente a partir de uma posição específica
    fn find_matching_brace_from(
        &self,
        chars: &[char],
        start: usize,
        open: char,
        close: char,
    ) -> Option<(usize, String)> {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut content = String::new();

        for i in start..chars.len() {
            let ch = chars[i];

            if escape_next {
                content.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                    content.push(ch);
                }
                '"' | '\'' => {
                    in_string = !in_string;
                    content.push(ch);
                }
                c if c == open && !in_string => {
                    depth += 1;
                    content.push(ch);
                }
                c if c == close && !in_string => {
                    if depth == 0 {
                        return Some((i, content));
                    } else {
                        depth -= 1;
                        content.push(ch);
                    }
                }
                _ => {
                    content.push(ch);
                }
            }
        }

        None
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
        let code = "{...a, b: 2, ...c}";
        let result = preprocessor.process(code);
        assert_eq!(result, "__spread_object([a, #{b: 2}, c])");
    }

    #[test]
    fn test_object_without_spread() {
        let preprocessor = SpreadPreprocessor::new();
        let code = "{a: 1, b: 2}";
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
        let code = "{...user, name: \"John\", ...settings, active: true}";
        let result = preprocessor.process(code);
        assert_eq!(
            result,
            "__spread_object([user, #{name: \"John\"}, settings, #{active: true}])"
        );
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
    fn test_object_literal_detection() {
        let preprocessor = SpreadPreprocessor::new();

        // Testa objetos literais simples
        let code = "{key: value}";
        let result = preprocessor.process(code);
        assert_eq!(result, "#{key: value}");

        // Testa objetos com múltiplas propriedades
        let code = "{name: \"John\", age: 30}";
        let result = preprocessor.process(code);
        assert_eq!(result, "#{name: \"John\", age: 30}");

        // Testa objeto vazio
        let code = "{}";
        let result = preprocessor.process(code);
        assert_eq!(result, "#{}");
    }

    #[test]
    fn test_object_literal_with_spread() {
        let preprocessor = SpreadPreprocessor::new();

        // Testa objeto com spread
        let code = "{...user, name: \"John\"}";
        let result = preprocessor.process(code);
        assert_eq!(result, "__spread_object([user, #{name: \"John\"}])");
    }

    #[test]
    fn test_code_blocks_not_converted() {
        let preprocessor = SpreadPreprocessor::new();

        // Bloco com declaração de variável
        let code = "{ let x = 1; x + 1 }";
        let result = preprocessor.process(code);
        assert_eq!(result, "{ let x = 1; x + 1 }");

        // Bloco com if
        let code = "{ if (x > 0) { return x; } }";
        let result = preprocessor.process(code);
        assert_eq!(result, "{ if (x > 0) { return x; } }");

        // Bloco com for loop
        let code = "{ for i in 0..10 { print(i); } }";
        let result = preprocessor.process(code);
        assert_eq!(result, "{ for i in 0..10 { print(i); } }");

        // Bloco com função
        let code = "{ fn test() { return 1; } }";
        let result = preprocessor.process(code);
        assert_eq!(result, "{ fn test() { return 1; } }");
    }

    #[test]
    fn test_mixed_objects_and_blocks() {
        let preprocessor = SpreadPreprocessor::new();

        // Objeto dentro de código
        let code = "{ let obj = {name: \"test\"}; obj }";
        let result = preprocessor.process(code);
        assert_eq!(result, "{ let obj = #{name: \"test\"}; obj }");

        // Múltiplos objetos
        let code = "let a = {x: 1}; let b = {y: 2};";
        let result = preprocessor.process(code);
        assert_eq!(result, "let a = #{x: 1}; let b = #{y: 2};");
    }

    #[test]
    fn test_nested_object_literals() {
        let preprocessor = SpreadPreprocessor::new();

        // Objetos aninhados
        let code = "{user: {name: \"John\", data: {age: 30}}}";
        let result = preprocessor.process(code);
        assert_eq!(result, "#{user: #{name: \"John\", data: #{age: 30}}}");
    }

    #[test]
    fn test_object_with_array_values() {
        let preprocessor = SpreadPreprocessor::new();

        // Objeto com arrays
        let code = "{items: [1, 2, 3], tags: [\"a\", \"b\"]}";
        let result = preprocessor.process(code);
        assert_eq!(result, "#{items: [1, 2, 3], tags: [\"a\", \"b\"]}");
    }

    #[test]
    fn test_complex_mixed_scenario() {
        let preprocessor = SpreadPreprocessor::new();

        // Cenário do test.phlow
        let code = r#"{
            let val = payload * 10;
            let no = [1, 2, 3];
            let obj = {target: 1};

            {
                item: val,
                ...obj,
                name: [...no,4,5,6],
                it: {a: 1, b: [2, ...no, ...obj]}
            }
        }"#;

        let result = preprocessor.process(code);

        // Verifica se objetos literais foram convertidos
        assert!(result.contains("#{target: 1}"));
        assert!(result.contains("__spread_object"));
        assert!(result.contains("__spread_array"));
        // Verifica se o bloco principal não foi convertido (contém let statements)
        assert!(!result.starts_with("#{"));
    }

    #[test]
    fn test_debug_test_phlow_case() {
        let preprocessor = SpreadPreprocessor::new();

        // Caso exato do test.phlow
        let code = r#"{
        let val = payload * 10;
        let no = [1, 2, 3];
        let obj = {target: 1};

        {
            item: val,
            ...obj,
            name: [...no,4,5,6],
            it: {a: 1, b: [2, ...no, ...obj]}
        }
      }"#;

        let result = preprocessor.process(code);
        println!("Input: {}", code);
        println!("Output: {}", result);

        // O bloco principal não deve ser convertido (contém múltiplas declarações let)
        assert!(!result.starts_with("#{"));

        // Verifica que não tem __spread_object aplicado ao bloco principal
        assert!(!result.starts_with("__spread_object"));
    }
}
