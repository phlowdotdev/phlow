use regex::Regex;
use rhai::{Engine, EvalAltResult};

pub fn build_functions() -> Engine {
    let mut engine = Engine::new();

    // Define operadores personalizados
    match engine.register_custom_operator("starts_with", 80) {
        Ok(engine) => engine.register_fn("start_withs", |x: String, y: String| x.starts_with(&y)),
        Err(_) => {
            panic!("Error on register custom operator starts_with");
        }
    };

    match engine.register_custom_operator("ends_with", 81) {
        Ok(engine) => engine.register_fn("ends_with", |x: String, y: String| x.ends_with(&y)),
        Err(_) => {
            panic!("Error on register custom operator ends_with");
        }
    };

    match engine.register_custom_operator("search", 82) {
        Ok(engine) => engine.register_fn("search", |x: String, y: String| match Regex::new(&x) {
            Ok(re) => re.is_match(&y),
            Err(_) => false,
        }),
        Err(_) => {
            panic!("Error on register custom operator search");
        }
    };

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
    fn test_custom_operators() {
        let engine = build_functions();

        let result: bool = engine.eval(r#""hello" starts_with "he""#).unwrap();
        assert!(result);

        let result: bool = engine.eval(r#""world" ends_with "ld""#).unwrap();
        assert!(result);

        let result: bool = engine.eval(r#""\\d+" search "123""#).unwrap();
        assert!(result);
    }

    #[test]
    fn test_merge_function() {
        let engine = build_functions();

        let result: rhai::Dynamic = engine
            .eval(r#"merge(#{ "a": 1, "b": 2 },#{ "b": 3, "c": 4 })"#)
            .unwrap();
        let map: rhai::Map = result.try_cast().unwrap();

        assert!(map.get("a").unwrap().as_int().unwrap() == 1);
        assert!(map.get("b").unwrap().as_int().unwrap() == 3);
        assert!(map.get("c").unwrap().as_int().unwrap() == 4);
    }
}
