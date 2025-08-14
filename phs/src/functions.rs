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
        } else {
            false
        }
    });

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
            s.chars().skip(start as usize).take((end - start) as usize).collect()
        }
    });

    match engine.register_custom_syntax(
        ["iff", "$expr$", "?", "$expr$", ":", "$expr$"],
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
            panic!("Error on register custom syntax iff");
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

    #[test]
    fn test_is_null_function() {
        let engine = build_functions();
        let result: bool = engine.eval(r#"is_null(null)"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#"is_null(123)"#).unwrap();
        assert!(!result);
        let result: bool = engine.eval(r#"is_null("texto")"#).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_is_not_null_function() {
        let engine = build_functions();
        let result: bool = engine.eval(r#"is_not_null(null)"#).unwrap();
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
        let result: bool = engine.eval(r#"is_empty(null)"#).unwrap();
        assert!(result);
    }

    #[test]
    fn test_merge_function() {
        let engine = build_functions();
        let result: rhai::Dynamic = engine.eval(r#"merge(#{ "a": 1, "b": 2 },#{ "b": 3, "c": 4 })"#).unwrap();
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
}
}
