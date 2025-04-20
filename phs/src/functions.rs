use regex::Regex;
use rhai::Engine;

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
}
