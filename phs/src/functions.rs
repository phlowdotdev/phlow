use base64::{engine::general_purpose, Engine as Base64Engine};
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::{DateTime, Utc};
use regex::Regex;
use rhai::{Engine, EvalAltResult};
use valu3::prelude::JsonMode;
use valu3::value::Value;

pub fn build_functions() -> Engine {
    let mut engine = Engine::new();
    // --- Funções de tempo globais ---
    // today() -> timestamp do início do dia UTC
    engine.register_fn("today", || {
        let now = Utc::now();
        Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap()
            .timestamp() as i64
    });
    // now() -> timestamp atual (segundos)
    engine.register_fn("now", || Utc::now().timestamp() as i64);
    // format(ts, fmt) -> string formatada
    engine.register_fn("format", |ts: i64, fmt: &str| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.format(fmt).to_string()
        } else {
            String::new()
        }
    });
    // diff(ts1, ts2) -> diferença em segundos
    engine.register_fn("diff", |ts1: i64, ts2: i64| ts1 - ts2);
    // add_days(ts, n)
    engine.register_fn("add_days", |ts: i64, n: i64| ts + n * 86400);
    // weekday(ts) -> 0=Domingo, 1=Segunda, ...
    engine.register_fn("weekday", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.weekday().num_days_from_sunday() as i64
        } else {
            -1i64
        }
    });
    // year(ts)
    engine.register_fn("year", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.year() as i64
        } else {
            0i64
        }
    });
    // month(ts)
    engine.register_fn("month", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.month() as i64
        } else {
            0i64
        }
    });
    // day(ts)
    engine.register_fn("day", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.day() as i64
        } else {
            0i64
        }
    });
    // hour(ts)
    engine.register_fn("hour", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.hour() as i64
        } else {
            0i64
        }
    });
    // minute(ts)
    engine.register_fn("minute", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.minute() as i64
        } else {
            0i64
        }
    });
    // second(ts)
    engine.register_fn("second", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.second() as i64
        } else {
            0i64
        }
    });
    // add_seconds(ts, n)
    engine.register_fn("add_seconds", |ts: i64, n: i64| ts + n);
    // add_minutes(ts, n)
    engine.register_fn("add_minutes", |ts: i64, n: i64| ts + n * 60);
    // add_hours(ts, n)
    engine.register_fn("add_hours", |ts: i64, n: i64| ts + n * 3600);
    // sub_seconds(ts, n)
    engine.register_fn("sub_seconds", |ts: i64, n: i64| ts - n);
    // sub_minutes(ts, n)
    engine.register_fn("sub_minutes", |ts: i64, n: i64| ts - n * 60);
    // from_iso(iso_str) -> timestamp (segundos)
    engine.register_fn("from_iso", |iso: &str| {
        DateTime::parse_from_rfc3339(iso)
            .map(|dt| dt.timestamp() as i64)
            .unwrap_or(0)
    });
    // to_iso(ts) -> string ISO
    engine.register_fn("to_iso", |ts: i64| {
        if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
            dt.to_rfc3339()
        } else {
            String::new()
        }
    });

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

    // Adiciona função starts_with para verificar se string começa com prefixo
    engine.register_fn("starts_with", |s: &str, prefix: &str| s.starts_with(prefix));

    // Adiciona função replace que retorna o valor alterado
    engine.register_fn("replace", |s: &str, target: &str, replacement: &str| {
        s.replace(target, replacement)
    });

    // Adiciona função slice para strings (com start e end)
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

    // Adiciona função slice para strings (apenas com start, vai até o final)
    engine.register_fn("slice", |s: &str, start: i64| {
        let len = s.chars().count() as i64;
        let start = if start < 0 {
            let abs_start = start.abs();
            if abs_start > len {
                0
            } else {
                len - abs_start
            }
        } else {
            start
        };

        if start >= len {
            String::new()
        } else {
            s.chars().skip(start as usize).collect()
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

    // Adiciona função to_url_encode
    engine.register_fn("to_url_encode", |s: &str| {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                b' ' => "+".to_string(),
                _ => format!("%{:02X}", b),
            })
            .collect::<String>()
    });

    // Adiciona função to_base64
    engine.register_fn("to_base64", |s: &str| {
        general_purpose::STANDARD.encode(s.as_bytes())
    });

    // Adiciona função base64_to_utf8 para decodificar Base64
    engine.register_fn("base64_to_utf8", |s: &str| -> String {
        // Adiciona padding se necessário
        let mut padded_input = s.to_string();
        let remainder = padded_input.len() % 4;
        if remainder != 0 {
            let padding_needed = 4 - remainder;
            for _ in 0..padding_needed {
                padded_input.push('=');
            }
        }

        match general_purpose::STANDARD.decode(&padded_input) {
            Ok(bytes) => {
                match String::from_utf8(bytes) {
                    Ok(decoded) => decoded,
                    Err(_) => String::new(), // Retorna string vazia se não for UTF-8 válido
                }
            }
            Err(_) => String::new(), // Retorna string vazia se não for Base64 válido
        }
    });

    // Adiciona função url_decode para decodificar URL encoding
    engine.register_fn("url_decode", |s: &str| -> String {
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '+' => result.push(b' '), // '+' representa espaço
                '%' => {
                    // Verifica se há dois caracteres hexadecimais após '%'
                    let hex1 = chars.next();
                    let hex2 = chars.next();

                    if let (Some(h1), Some(h2)) = (hex1, hex2) {
                        let hex_str = format!("{}{}", h1, h2);
                        if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                            result.push(byte);
                        } else {
                            // Se não for hex válido, adiciona os caracteres literalmente
                            result.extend(format!("%{}{}", h1, h2).bytes());
                        }
                    } else {
                        // Se não há caracteres suficientes, adiciona '%' literal
                        result.push(b'%');
                        if let Some(h1) = hex1 {
                            result.extend(h1.to_string().bytes());
                        }
                        if let Some(h2) = hex2 {
                            result.extend(h2.to_string().bytes());
                        }
                    }
                }
                _ => {
                    // Caracteres normais são adicionados como UTF-8
                    result.extend(ch.to_string().bytes());
                }
            }
        }

        // Converte bytes para string UTF-8
        match String::from_utf8(result) {
            Ok(decoded) => decoded,
            Err(_) => String::new(), // Retorna string vazia se não for UTF-8 válido
        }
    });

    engine.register_fn("parse", |s: &str| -> rhai::Dynamic {
        match Value::json_to_value(s) {
            Ok(value) => {
                match value {
                    Value::Null => rhai::Dynamic::UNIT,
                    Value::Boolean(b) => rhai::Dynamic::from(b),
                    Value::Number(n) => {
                        let num_str = n.to_string();
                        if num_str.contains('.') {
                            // Float
                            num_str
                                .parse::<f64>()
                                .map(|f| rhai::Dynamic::from(f))
                                .unwrap_or_else(|_| rhai::Dynamic::from(num_str))
                        } else {
                            // Integer
                            num_str
                                .parse::<i64>()
                                .map(|i| rhai::Dynamic::from(i))
                                .unwrap_or_else(|_| rhai::Dynamic::from(num_str))
                        }
                    }
                    Value::String(s) => rhai::Dynamic::from(s.to_string()),
                    Value::Array(_) => {
                        // Para arrays, tentamos usar to_dynamic do rhai::serde
                        let json_str = value.to_string();
                        match rhai::serde::to_dynamic(&value) {
                            Ok(dynamic_val) => dynamic_val,
                            Err(_) => rhai::Dynamic::from(json_str),
                        }
                    }
                    Value::Object(_) => {
                        // Para objetos, tentamos usar to_dynamic do rhai::serde
                        let json_str = value.to_string();
                        match rhai::serde::to_dynamic(&value) {
                            Ok(dynamic_val) => dynamic_val,
                            Err(_) => rhai::Dynamic::from(json_str),
                        }
                    }
                    // Para outros tipos (Undefined, DateTime), convertemos para string
                    _ => rhai::Dynamic::from(value.to_string()),
                }
            }
            Err(_) => rhai::Dynamic::UNIT,
        }
    });

    // Função to_json para converter objetos Rhai em string JSON
    engine.register_fn("to_json", |value: rhai::Dynamic| -> rhai::Dynamic {
        let value: Value = match rhai::serde::from_dynamic(&value) {
            Ok(serde_value) => serde_value,
            Err(_) => Value::Undefined,
        };

        let json = value.to_json(JsonMode::Inline);

        rhai::Dynamic::from(json)
    });

    // Funções para spread de objetos e arrays
    engine.register_fn("__spread_object", |objects: rhai::Array| -> rhai::Dynamic {
        let mut result_map = rhai::Map::new();

        for obj in objects {
            if let Some(map) = obj.try_cast::<rhai::Map>() {
                for (key, value) in map {
                    result_map.insert(key, value);
                }
            }
        }

        rhai::Dynamic::from(result_map)
    });

    engine.register_fn("__spread_array", |arrays: rhai::Array| -> rhai::Dynamic {
        let mut result_array = rhai::Array::new();

        for arr in arrays {
            if let Some(array) = arr.clone().try_cast::<rhai::Array>() {
                result_array.extend(array);
            } else {
                // Se não for array, adiciona como elemento único
                result_array.push(arr);
            }
        }

        rhai::Dynamic::from(result_array)
    });

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
    #[test]
    fn test_time_extras() {
        let engine = build_functions();
        // today
        let today: i64 = engine.eval("today()").unwrap();
        let now: i64 = engine.eval("now()").unwrap();
        assert!(now >= today);
        assert!(now - today < 86400);

        // format
        let iso: String = engine.eval("to_iso(1692362096)").unwrap();
        assert_eq!(iso, "2023-08-18T12:34:56+00:00");
        let custom: String = engine
            .eval(r#"format(1692362096, "%d/%m/%Y %H:%M:%S")"#)
            .unwrap();
        assert_eq!(custom, "18/08/2023 12:34:56");

        // diff
        assert_eq!(engine.eval::<i64>("diff(100, 50)").unwrap(), 50);

        // add_days
        assert_eq!(
            engine.eval::<i64>("add_days(1000, 2)").unwrap(),
            1000 + 2 * 86400
        );

        // weekday
        let wd: i64 = engine.eval("weekday(1692362096)").unwrap();
        assert_eq!(wd, 5); // 2023-08-18 é sexta-feira

        // year, month, day, hour, minute, second
        assert_eq!(engine.eval::<i64>("year(1692362096)").unwrap(), 2023);
        assert_eq!(engine.eval::<i64>("month(1692362096)").unwrap(), 8);
        assert_eq!(engine.eval::<i64>("day(1692362096)").unwrap(), 18);
        assert_eq!(engine.eval::<i64>("hour(1692362096)").unwrap(), 12);
        assert_eq!(engine.eval::<i64>("minute(1692362096)").unwrap(), 34);
        assert_eq!(engine.eval::<i64>("second(1692362096)").unwrap(), 56);
    }
    use super::*;

    #[test]
    fn test_time_functions() {
        let engine = build_functions();
        // now deve retornar algo próximo do tempo atual
        let now: i64 = engine.eval("now()").unwrap();
        let sys_now = Utc::now().timestamp();
        assert!((now - sys_now).abs() < 5); // tolerância de 5s

        // add_seconds
        assert_eq!(engine.eval::<i64>("add_seconds(1000, 10)").unwrap(), 1010);
        // add_minutes
        assert_eq!(engine.eval::<i64>("add_minutes(1000, 2)").unwrap(), 1120);
        // add_hours
        assert_eq!(engine.eval::<i64>("add_hours(1000, 1)").unwrap(), 4600);
        // sub_seconds
        assert_eq!(engine.eval::<i64>("sub_seconds(1000, 10)").unwrap(), 990);
        // sub_minutes
        assert_eq!(engine.eval::<i64>("sub_minutes(1000, 2)").unwrap(), 880);

        // from_iso
        let ts: i64 = engine.eval(r#"from_iso("2023-08-18T12:34:56Z")"#).unwrap();
        assert_eq!(ts, 1692362096);

        // to_iso
        let iso: String = engine.eval("to_iso(1692362096)").unwrap();
        assert!(iso.starts_with("2023-08-18T12:34:56"));
    }

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
    fn test_starts_with_function() {
        let engine = build_functions();

        // Teste básico: string começa com prefixo
        let result: bool = engine
            .eval(r#""Bearer token123".starts_with("Bearer")"#)
            .unwrap();
        assert!(result);

        // Teste com espaço: string começa com prefixo incluindo espaço
        let result: bool = engine
            .eval(r#""Bearer token123".starts_with("Bearer ")"#)
            .unwrap();
        assert!(result);

        // Teste negativo: string não começa com prefixo
        let result: bool = engine
            .eval(r#""Basic auth123".starts_with("Bearer")"#)
            .unwrap();
        assert!(!result);

        // Teste com string vazia como prefixo (deve retornar true)
        let result: bool = engine.eval(r#""qualquer texto".starts_with("")"#).unwrap();
        assert!(result);

        // Teste com prefixo maior que a string (deve retornar false)
        let result: bool = engine.eval(r#""abc".starts_with("abcdef")"#).unwrap();
        assert!(!result);

        // Teste case-sensitive
        let result: bool = engine.eval(r#""Bearer".starts_with("bearer")"#).unwrap();
        assert!(!result);

        // Teste com caracteres especiais
        let result: bool = engine.eval(r#""@user123".starts_with("@")"#).unwrap();
        assert!(result);
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
        let result: bool = engine.eval(r#""".is_empty()"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#""   ".is_empty()"#).unwrap();
        assert!(result);
        let result: bool = engine.eval(r#""abc".is_empty()"#).unwrap();
        assert!(!result);
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
        let result: String = engine.eval(r#""abcdef".slice(0,3)"#).unwrap();
        assert_eq!(result, "abc");
        let result: String = engine.eval(r#""abcdef".slice(3)"#).unwrap();
        assert_eq!(result, "def");
        let result: String = engine.eval(r#""abcdef".slice(-2)"#).unwrap();
        assert_eq!(result, "ef");
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
        let result: String = engine
            .eval(r#"let a = #{value: "ábc"}; a.value.capitalize()"#)
            .unwrap();
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
    fn test_to_url_encode() {
        let engine = build_functions();

        // Teste básico com espaços
        assert_eq!(
            engine
                .eval::<String>(r#""Hello World".to_url_encode()"#)
                .unwrap(),
            "Hello+World"
        );

        // Teste com caracteres especiais
        assert_eq!(
            engine
                .eval::<String>(r#""user@example.com".to_url_encode()"#)
                .unwrap(),
            "user%40example.com"
        );

        // Teste com caracteres que não precisam ser codificados
        assert_eq!(
            engine
                .eval::<String>(r#""abc-123_test.file~".to_url_encode()"#)
                .unwrap(),
            "abc-123_test.file~"
        );

        // Teste com caracteres acentuados (UTF-8 de 1 byte)
        assert_eq!(
            engine
                .eval::<String>(r#""café & maçã".to_url_encode()"#)
                .unwrap(),
            "caf%C3%A9+%26+ma%C3%A7%C3%A3"
        );

        // Teste string vazia
        assert_eq!(engine.eval::<String>(r#""".to_url_encode()"#).unwrap(), "");
    }

    #[test]
    fn test_to_base64() {
        let engine = build_functions();

        // Teste básico
        assert_eq!(
            engine
                .eval::<String>(r#""Hello World".to_base64()"#)
                .unwrap(),
            "SGVsbG8gV29ybGQ="
        );

        // Teste com string vazia
        assert_eq!(engine.eval::<String>(r#""".to_base64()"#).unwrap(), "");

        // Teste com caracteres especiais
        assert_eq!(
            engine
                .eval::<String>(r#""user@example.com".to_base64()"#)
                .unwrap(),
            "dXNlckBleGFtcGxlLmNvbQ=="
        );

        // Teste com caracteres acentuados
        assert_eq!(
            engine.eval::<String>(r#""café".to_base64()"#).unwrap(),
            "Y2Fmw6k="
        );

        // Teste com números
        assert_eq!(
            engine.eval::<String>(r#""12345".to_base64()"#).unwrap(),
            "MTIzNDU="
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

    #[test]
    fn test_base64_to_utf8() {
        let engine = build_functions();

        // Teste básico - decodifica "Hello World"
        assert_eq!(
            engine
                .eval::<String>(r#""SGVsbG8gV29ybGQ=".base64_to_utf8()"#)
                .unwrap(),
            "Hello World"
        );

        // Teste com string vazia (Base64 válido)
        assert_eq!(engine.eval::<String>(r#""".base64_to_utf8()"#).unwrap(), "");

        // Teste com caracteres especiais
        assert_eq!(
            engine
                .eval::<String>(r#""dXNlckBleGFtcGxlLmNvbQ==".base64_to_utf8()"#)
                .unwrap(),
            "user@example.com"
        );

        // Teste com caracteres acentuados
        assert_eq!(
            engine
                .eval::<String>(r#""Y2Fmw6k=".base64_to_utf8()"#)
                .unwrap(),
            "café"
        );

        // Teste com números
        assert_eq!(
            engine
                .eval::<String>(r#""MTIzNDU=".base64_to_utf8()"#)
                .unwrap(),
            "12345"
        );

        // Teste com Base64 inválido - deve retornar string vazia
        assert_eq!(
            engine
                .eval::<String>(r#""invalid_base64!@#".base64_to_utf8()"#)
                .unwrap(),
            ""
        );

        // Teste com Base64 válido mas não UTF-8 - deve retornar string vazia
        // (Criando um caso onde temos bytes válidos de Base64 mas que não formam UTF-8 válido)
        assert_eq!(
            engine.eval::<String>(r#""//8=".base64_to_utf8()"#).unwrap(),
            ""
        );

        // Teste com JWT válido em Base64 - deve decodificar corretamente
        assert_eq!(
            engine
                .eval::<String>(r#""eyJlbWFpbCI6ImV4YW1wbGVAZXhhbXBsZS5jb20ifQ".base64_to_utf8()"#)
                .unwrap(),
            "{\"email\":\"example@example.com\"}"
        );
    }

    #[test]
    fn test_url_decode() {
        let engine = build_functions();

        // Teste básico com espaços (representados por +)
        assert_eq!(
            engine
                .eval::<String>(r#""Hello+World".url_decode()"#)
                .unwrap(),
            "Hello World"
        );

        // Teste com caracteres especiais codificados
        assert_eq!(
            engine
                .eval::<String>(r#""user%40example.com".url_decode()"#)
                .unwrap(),
            "user@example.com"
        );

        // Teste com caracteres que não precisam decodificação
        assert_eq!(
            engine
                .eval::<String>(r#""abc-123_test.file~".url_decode()"#)
                .unwrap(),
            "abc-123_test.file~"
        );

        // Teste com caracteres acentuados codificados em UTF-8
        assert_eq!(
            engine
                .eval::<String>(r#""caf%C3%A9+%26+ma%C3%A7%C3%A3".url_decode()"#)
                .unwrap(),
            "café & maçã"
        );

        // Teste string vazia
        assert_eq!(engine.eval::<String>(r#""".url_decode()"#).unwrap(), "");

        // Teste com % sem códigos hex válidos (deve manter literal)
        assert_eq!(
            engine.eval::<String>(r#""%ZZ".url_decode()"#).unwrap(),
            "%ZZ"
        );

        // Teste com % no final da string
        assert_eq!(
            engine.eval::<String>(r#""test%".url_decode()"#).unwrap(),
            "test%"
        );

        // Teste com % seguido de apenas um caractere
        assert_eq!(
            engine.eval::<String>(r#""test%2".url_decode()"#).unwrap(),
            "test%2"
        );

        // Teste complexo misturando diferentes tipos de codificação
        assert_eq!(
            engine
                .eval::<String>(r#""Ol%C3%A1+mundo%21+Como+vai%3F".url_decode()"#)
                .unwrap(),
            "Olá mundo! Como vai?"
        );
    }

    #[test]
    fn test_parse() {
        let engine = build_functions();

        // Teste com string JSON válida (objeto) - deve retornar um Map do Rhai
        let result: rhai::Dynamic = engine
            .eval(r#""{\"name\":\"João\",\"age\":30}".parse()"#)
            .unwrap();

        // Verifica se é um Map
        if let Some(map) = result.clone().try_cast::<rhai::Map>() {
            // Verifica se contém as chaves esperadas
            assert!(map.contains_key("name"));
            assert!(map.contains_key("age"));
            // Verifica os valores
            if let Some(name) = map.get("name") {
                if let Some(name_str) = name.clone().try_cast::<String>() {
                    assert_eq!(name_str, "João");
                }
            }
            if let Some(age) = map.get("age") {
                if let Ok(age_val) = age.as_int() {
                    assert_eq!(age_val, 30);
                }
            }
        } else {
            panic!("Esperado um Map, mas recebeu: {:?}", result.type_name());
        }

        // Teste com string JSON válida (array) - deve retornar um Array do Rhai
        let result: rhai::Dynamic = engine.eval(r#""[1, 2, 3, \"test\"]".parse()"#).unwrap();

        // Verifica se é um Array
        if let Some(array) = result.clone().try_cast::<rhai::Array>() {
            assert_eq!(array.len(), 4);
            // Verifica os valores
            assert_eq!(array[0].as_int().unwrap(), 1);
            assert_eq!(array[1].as_int().unwrap(), 2);
            assert_eq!(array[2].as_int().unwrap(), 3);
            assert_eq!(array[3].clone().try_cast::<String>().unwrap(), "test");
        } else {
            panic!("Esperado um Array, mas recebeu: {:?}", result.type_name());
        }

        // Teste com string JSON válida (string)
        let result: String = engine.eval(r#""\"hello world\"".parse()"#).unwrap();
        assert_eq!(result, "hello world");

        // Teste com número JSON
        let result: i64 = engine.eval(r#""42".parse()"#).unwrap();
        assert_eq!(result, 42);

        // Teste com float JSON
        let result: f64 = engine.eval(r#""3.14".parse()"#).unwrap();
        assert_eq!(result, 3.14);

        // Teste com boolean JSON true
        let result: bool = engine.eval(r#""true".parse()"#).unwrap();
        assert_eq!(result, true);

        // Teste com boolean JSON false
        let result: bool = engine.eval(r#""false".parse()"#).unwrap();
        assert_eq!(result, false);

        // Teste com null JSON
        let result: rhai::Dynamic = engine.eval(r#""null".parse()"#).unwrap();
        assert!(result.is_unit());

        // Teste com JSON inválido - deve retornar unit (null)
        let result: rhai::Dynamic = engine.eval(r#""{invalid json}".parse()"#).unwrap();
        assert!(result.is_unit());

        // Teste com string vazia
        let result: rhai::Dynamic = engine.eval(r#""".parse()"#).unwrap();
        assert!(result.is_unit());
    }

    #[test]
    fn test_parse_complex_structures() {
        let engine = build_functions();

        // Teste com objeto JSON aninhado
        let result: rhai::Dynamic = engine
            .eval(r#""{\"user\":{\"name\":\"Maria\",\"age\":25},\"active\":true}".parse()"#)
            .unwrap();

        if let Some(map) = result.clone().try_cast::<rhai::Map>() {
            assert!(map.contains_key("user"));
            assert!(map.contains_key("active"));

            // Testa acesso ao objeto aninhado
            if let Some(user) = map.get("user") {
                if let Some(user_map) = user.clone().try_cast::<rhai::Map>() {
                    assert!(user_map.contains_key("name"));
                    assert!(user_map.contains_key("age"));
                }
            }
        } else {
            panic!("Esperado um Map para objeto aninhado");
        }

        // Teste com array de objetos
        let result: rhai::Dynamic = engine
            .eval(r#""[{\"id\":1,\"name\":\"João\"},{\"id\":2,\"name\":\"Maria\"}]".parse()"#)
            .unwrap();

        if let Some(array) = result.clone().try_cast::<rhai::Array>() {
            assert_eq!(array.len(), 2);

            // Verifica o primeiro objeto do array
            if let Some(first_obj) = array.get(0) {
                if let Some(obj_map) = first_obj.clone().try_cast::<rhai::Map>() {
                    assert!(obj_map.contains_key("id"));
                    assert!(obj_map.contains_key("name"));
                }
            }
        } else {
            panic!("Esperado um Array de objetos");
        }
    }

    #[test]
    fn test_base64_to_utf8_and_parse() {
        let engine = build_functions();

        let result: String = engine
            .eval(
                r#""eyJlbWFpbCI6ImV4YW1wbGVAZXhhbXBsZS5jb20ifQ==".base64_to_utf8().parse().email"#,
            )
            .unwrap();

        assert_eq!(result, "example@example.com");
    }

    #[test]
    fn test_to_json() {
        let engine = build_functions();

        // Teste com string simples
        let result: String = engine.eval(r#""hello world".to_json()"#).unwrap();
        assert_eq!(result, "\"hello world\"");

        // Teste com número inteiro
        let result: String = engine.eval(r#"42.to_json()"#).unwrap();
        assert_eq!(result, "42");

        // Teste com número float
        let result: String = engine.eval(r#"3.14.to_json()"#).unwrap();
        assert_eq!(result, "3.14");

        // Teste com boolean true
        let result: String = engine.eval(r#"true.to_json()"#).unwrap();
        assert_eq!(result, "true");

        // Teste com boolean false
        let result: String = engine.eval(r#"false.to_json()"#).unwrap();
        assert_eq!(result, "false");

        // Teste com null/unit
        let result: String = engine.eval(r#"().to_json()"#).unwrap();
        assert_eq!(result, "null");

        // Teste com objeto simples
        let result: String = engine
            .eval(r#"#{name: "João", age: 30}.to_json()"#)
            .unwrap();
        // Como a ordem das chaves pode variar, vamos verificar se contém as partes esperadas
        assert!(result.contains("\"name\":\"João\""));
        assert!(result.contains("\"age\":30"));
        assert!(result.starts_with("{"));
        assert!(result.ends_with("}"));

        // Teste com array simples
        let result: String = engine.eval(r#"[1, 2, "test", true].to_json()"#).unwrap();
        assert_eq!(result, "[1,2,\"test\",true]");

        // Teste com array vazio
        let result: String = engine.eval(r#"[].to_json()"#).unwrap();
        assert_eq!(result, "[]");

        // Teste com objeto vazio
        let result: String = engine.eval(r#"#{}.to_json()"#).unwrap();
        assert_eq!(result, "{}");

        // Teste com string contendo aspas
        let result: String = engine.eval(r#""He said \"hello\"".to_json()"#).unwrap();
        assert_eq!(result, "\"He said \\\"hello\\\"\"");
    }
}
