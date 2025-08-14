use regex::Regex;
use rhai::{Engine, EvalAltResult};

pub fn build_functions() -> Engine {
    let mut engine = Engine::new();

    engine.register_fn("is_null", |x: rhai::Dynamic| x.is_unit());

    engine.register_fn("is_not_null", |x: rhai::Dynamic| !x.is_unit());

    engine.register_fn("merge", |x: rhai::Dynamic, y: rhai::Dynamic| {
        if let (Some(mut x), Some(y)) = (x.try_cast::<rhai::Map>(), y.try_cast::<rhai::Map>()) {
            x.extend(y.into_iter());
            rhai::Dynamic::from(x)
        } else {
            rhai::Dynamic::UNIT
        }
    });

    engine.register_fn("is_empty", |x: rhai::Dynamic| {
        if x.is_unit() {
            true
        } else if let Some(s) = x.clone().try_cast::<String>() {
            s.trim().is_empty()
        } else if let Some(s) = x.clone().try_cast::<rhai::ImmutableString>() {
            s.trim().is_empty()
        } else {
            false
        }
    });

    // Registra is_empty especificamente para strings também
    engine.register_fn("is_empty", |s: &str| s.trim().is_empty());

    // Adiciona função search como método para String
    engine.register_fn("search", |s: &str, pattern: &str| {
        Regex::new(pattern)
            .map(|re| re.is_match(s))
            .unwrap_or(false)
    });

    // Adiciona função replace que retorna o valor alterado
    engine.register_fn("replace", |s: &str, target: &str, replacement: &str| {
        s.replace(target, replacement)
    });

    // Adiciona função slice para strings
    engine.register_fn("slice", |s: &str, start: i64, end: i64| {
        let len = s.chars().count() as i64;
        let start = if start < 0 { 0 } else { start };
        let end = if end > len { len } else { end };
        if start >= end || start >= len {
            String::new()
        } else {
            s.chars()
                .skip(start as usize)
                .take((end - start) as usize)
                .collect()
        }
    });

    // Adiciona função capitalize
    engine.register_fn("capitalize", |s: &str| {
        if s.is_empty() {
            String::new()
        } else {
            let mut chars: Vec<char> = s.chars().collect();
            chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
            chars.iter().collect()
        }
    });

    // Função helper para dividir palavras
    fn split_words(s: &str) -> Vec<String> {
        let mut words = Vec::new();

        // Primeiro, substitui separadores por espaços
        let normalized = s.replace("_", " ").replace("-", " ");

        // Depois processa camelCase e PascalCase
        let camel_regex = Regex::new(r"([a-z])([A-Z])").unwrap();
        let spaced = camel_regex.replace_all(&normalized, "$1 $2");

        // Regex simples para capturar palavras (sem lookahead)
        let word_regex = Regex::new(r"[a-zA-Z]+|[0-9]+").unwrap();

        // Extrai palavras
        for word_match in word_regex.find_iter(&spaced) {
            let word = word_match.as_str().to_lowercase();
            if !word.is_empty() {
                words.push(word);
            }
        }

        // Se não encontrou palavras, usa a string inteira
        if words.is_empty() && !s.trim().is_empty() {
            words.push(s.trim().to_lowercase());
        }

        words
    }

    // Adiciona função to_snake_case
    engine.register_fn("to_snake_case", |s: &str| split_words(s).join("_"));

    // Adiciona função to_camel_case
    engine.register_fn("to_camel_case", |s: &str| {
        let words = split_words(s);
        if words.is_empty() {
            return String::new();
        }

        let mut result = words[0].clone();
        for word in words.iter().skip(1) {
            if !word.is_empty() {
                let mut chars: Vec<char> = word.chars().collect();
                chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                result.push_str(&chars.iter().collect::<String>());
            }
        }
        result
    });

    // Adiciona função to_kebab_case
    engine.register_fn("to_kebab_case", |s: &str| split_words(s).join("-"));

    match engine.register_custom_syntax(
        ["when", "$expr$", "?", "$expr$", ":", "$expr$"],
        false,
        |context, inputs| match context.eval_expression_tree(&inputs[0])?.as_bool() {
            Ok(true) => context.eval_expression_tree(&inputs[1]),
            Ok(false) => context.eval_expression_tree(&inputs[2]),
            Err(typ) => Err(Box::new(EvalAltResult::ErrorMismatchDataType(
                "bool".to_string(),
                typ.to_string(),
                inputs[0].position(),
            ))),
        },
    ) {
        Ok(engine) => engine,
        Err(_) => {
            panic!("Error on register custom syntax when");
        }
    };

    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_function() {
        let engine = build_functions();

        // Teste simples: texto contém substring
        let result: bool = engine.eval(r#""meu texto".search("texto")"#).unwrap();
        assert!(result);

        // Teste regex: texto começa com "meu"
        let result: bool = engine.eval(r#""meu texto".search("^meu")"#).unwrap();
        assert!(result);

        // Teste regex: texto termina com "texto"
        let result: bool = engine.eval(r#""meu texto".search("texto$")"#).unwrap();
        assert!(result);

        // Teste negativo: não contém "abc"
        let result: bool = engine.eval(r#""meu texto".search("abc")"#).unwrap();
        assert!(!result);

        // Teste regex inválido: deve retornar false
        let result: bool = engine.eval(r#""meu texto".search("[")"#).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_replace_function() {
        let engine = build_functions();

        // Substituição simples
        let result: String = engine
            .eval(r#""meu texto".replace("texto", "valor")"#)
            .unwrap();
        assert_eq!(result, "meu valor");

        // Substituição sem ocorrência
        let result: String = engine
            .eval(r#""meu texto".replace("abc", "valor")"#)
            .unwrap();
        assert_eq!(result, "meu texto");

        // Substituição múltipla
        let result: String = engine.eval(r#""abc abc abc".replace("abc", "x")"#).unwrap();
        assert_eq!(result, "x x x");

        // Substituição por vazio
        let result: String = engine.eval(r#""meu texto".replace("texto", "")"#).unwrap();
        assert_eq!(result, "meu ");
    }

    #[test]
    fn test_is_null_function() {
        let engine = build_functions();
        let result: bool = engine.eval(r#"is_null(())"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#"is_null(123)"#).unwrap();
        assert!(!result);
        let result: bool = engine.eval(r#"is_null("texto")"#).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_is_not_null_function() {
        let engine = build_functions();
        let result: bool = engine.eval(r#"is_not_null(())"#).unwrap();
        assert!(!result);
        let result: bool = engine.eval(r#"is_not_null(123)"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#"is_not_null("texto")"#).unwrap();
        assert!(result);
    }

    #[test]
    fn test_is_empty_function() {
        let engine = build_functions();
        let result: bool = engine.eval(r#"is_empty("")"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#"is_empty("   ")"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#"is_empty("abc")"#).unwrap();
        assert!(!result);
        let result: bool = engine.eval(r#"is_empty(())"#).unwrap();
        assert!(result);
    }

    #[test]
    fn test_merge_function() {
        let engine = build_functions();
        let result: rhai::Dynamic = engine
            .eval(r#"merge(#{ "a": 1, "b": 2 },#{ "b": 3, "c": 4 })"#)
            .unwrap();
        let map: rhai::Map = result.try_cast().unwrap();
        assert_eq!(map.get("a").unwrap().as_int().unwrap(), 1);
        assert_eq!(map.get("b").unwrap().as_int().unwrap(), 3);
        assert_eq!(map.get("c").unwrap().as_int().unwrap(), 4);
    }

    #[test]
    fn test_slice_function() {
        let engine = build_functions();
        let result: String = engine.eval(r#""abcdef".slice(1, 4)"#).unwrap();
        assert_eq!(result, "bcd");
        let result: String = engine.eval(r#""abcdef".slice(-2, 3)"#).unwrap();
        assert_eq!(result, "abc");
        let result: String = engine.eval(r#""abcdef".slice(2, 10)"#).unwrap();
        assert_eq!(result, "cdef");
        let result: String = engine.eval(r#""abcdef".slice(4, 2)"#).unwrap();
        assert_eq!(result, "");
        let result: String = engine.eval(r#""abcdef".slice(10, 12)"#).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_capitalize_function() {
        let engine = build_functions();
        let result: String = engine.eval(r#""exemplo".capitalize()"#).unwrap();
        assert_eq!(result, "Exemplo");
        let result: String = engine.eval(r#""a".capitalize()"#).unwrap();
        assert_eq!(result, "A");
        let result: String = engine.eval(r#""".capitalize()"#).unwrap();
        assert_eq!(result, "");
        let result: String = engine.eval(r#""ábc".capitalize()"#).unwrap();
        assert_eq!(result, "Ábc");
    }

    #[test]
    fn test_to_snake_case() {
        let engine = build_functions();
        assert_eq!(
            engine
                .eval::<String>(r#""Meu texto exemplo".to_snake_case()"#)
                .unwrap(),
            "meu_texto_exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meuTextoExemplo".to_snake_case()"#)
                .unwrap(),
            "meu_texto_exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""MeuTextoExemplo".to_snake_case()"#)
                .unwrap(),
            "meu_texto_exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meu_texto_exemplo".to_snake_case()"#)
                .unwrap(),
            "meu_texto_exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meu-texto-exemplo".to_snake_case()"#)
                .unwrap(),
            "meu_texto_exemplo"
        );
    }

    #[test]
    fn test_to_camel_case() {
        let engine = build_functions();
        assert_eq!(
            engine
                .eval::<String>(r#""Meu texto exemplo".to_camel_case()"#)
                .unwrap(),
            "meuTextoExemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meu_texto_exemplo".to_camel_case()"#)
                .unwrap(),
            "meuTextoExemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meu-texto-exemplo".to_camel_case()"#)
                .unwrap(),
            "meuTextoExemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""MeuTextoExemplo".to_camel_case()"#)
                .unwrap(),
            "meuTextoExemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meuTextoExemplo".to_camel_case()"#)
                .unwrap(),
            "meuTextoExemplo"
        );
    }

    #[test]
    fn test_to_kebab_case() {
        let engine = build_functions();
        assert_eq!(
            engine
                .eval::<String>(r#""Meu texto exemplo".to_kebab_case()"#)
                .unwrap(),
            "meu-texto-exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meuTextoExemplo".to_kebab_case()"#)
                .unwrap(),
            "meu-texto-exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meu_texto_exemplo".to_kebab_case()"#)
                .unwrap(),
            "meu-texto-exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""MeuTextoExemplo".to_kebab_case()"#)
                .unwrap(),
            "meu-texto-exemplo"
        );
        assert_eq!(
            engine
                .eval::<String>(r#""meu-texto-exemplo".to_kebab_case()"#)
                .unwrap(),
            "meu-texto-exemplo"
        );
    }

    #[test]
    fn test_when_ternary() {
        let engine = build_functions();

        // Teste quando condição é verdadeira
        let result: i64 = engine.eval(r#"when true ? 42 : 0"#).unwrap();
        assert_eq!(result, 42);

        // Teste quando condição é falsa
        let result: i64 = engine.eval(r#"when false ? 42 : 0"#).unwrap();
        assert_eq!(result, 0);

        // Teste com expressão condicional
        let result: String = engine.eval(r#"when 5 > 3 ? "maior" : "menor""#).unwrap();
        assert_eq!(result, "maior");

        // Teste com strings
        let result: String = engine
            .eval(r#"when "abc".search("b") ? "encontrou" : "não encontrou""#)
            .unwrap();
        assert_eq!(result, "encontrou");
    }
}
