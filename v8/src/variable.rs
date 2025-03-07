use regex::Regex;
use valu3::{prelude::StringBehavior, value::Value};

pub struct Variable {
    value: Value,
}

impl Variable {
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &Value {
        &self.value
    }

    pub fn equal(&self, other: &Variable) -> bool {
        self.value == other.value
    }

    pub fn greater_than(&self, other: &Variable) -> bool {
        self.value > other.value
    }

    pub fn less_than(&self, other: &Variable) -> bool {
        self.value < other.value
    }

    pub fn greater_than_or_equal(&self, other: &Variable) -> bool {
        self.value >= other.value
    }

    pub fn less_than_or_equal(&self, other: &Variable) -> bool {
        self.value <= other.value
    }

    pub fn contains(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let left = self.get().as_str();
            let right = other.get().as_str();

            return left.contains(right);
        } else if self.get().is_array() && other.get().is_array() {
            let left = match self.get().as_array() {
                Some(array) => array,
                None => return false,
            };
            let right = match other.get().as_array() {
                Some(array) => array,
                None => return false,
            };

            return left.into_iter().any(|x| right.into_iter().any(|y| x == y));
        }

        return false;
    }

    pub fn starts_with(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let left = self.get().as_str();
            let right = other.get().as_str();

            return left.starts_with(right);
        }

        return false;
    }

    pub fn ends_with(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let left = self.get().as_str();
            let right = other.get().as_str();

            return left.ends_with(right);
        }

        return false;
    }

    pub fn regex(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let left = self.get().as_str();
            let right = other.get().as_str();

            return Regex::new(right).unwrap().is_match(left);
        }

        return false;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use valu3::value::Value;

    #[test]
    fn test_variable_get() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert_eq!(variable.get(), &value);
    }

    #[test]
    fn test_variable_equal() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert!(variable.equal(&Variable::new(value.clone())));
    }

    #[test]
    fn test_variable_not_equal() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert!(!variable.equal(&Variable::new(value.clone())));
    }

    #[test]
    fn test_variable_greater_than() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert!(variable.greater_than(&Variable::new(Value::from(5i64))));
    }

    #[test]
    fn test_variable_less_than() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert!(variable.less_than(&Variable::new(Value::from(20i64))));
    }

    #[test]
    fn test_variable_greater_than_or_equal() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert!(variable.greater_than_or_equal(&Variable::new(Value::from(10i64))));
    }

    #[test]
    fn test_variable_less_than_or_equal() {
        let value = Value::from(10i64);
        let variable = Variable::new(value.clone());

        assert!(variable.less_than_or_equal(&Variable::new(Value::from(10i64))));
    }

    #[test]
    fn test_variable_array_contains() {
        let value = Value::from(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]);
        let variable = Variable::new(value.clone());
        let expected = Variable::new(Value::from(vec![Value::from(2i64)]));

        assert!(variable.contains(&expected));
    }

    #[test]
    fn test_variable_string_contains() {
        let value = Value::from("hello");
        let variable = Variable::new(value.clone());
        let expected = Variable::new(Value::from("ell"));

        assert!(variable.contains(&expected));
    }

    #[test]
    fn test_variable_string_starts_with() {
        let value = Value::from("hello");
        let variable = Variable::new(value.clone());
        let expected = Variable::new(Value::from("he"));

        assert!(variable.starts_with(&expected));
    }

    #[test]
    fn test_variable_string_ends_with() {
        let value = Value::from("hello");
        let variable = Variable::new(value.clone());
        let expected = Variable::new(Value::from("lo"));

        assert!(variable.ends_with(&expected));
    }

    #[test]
    fn test_variable_string_regex() {
        let value = Value::from("hello");
        let variable = Variable::new(value.clone());
        let expected = Variable::new(Value::from("h.*o"));

        assert!(variable.regex(&expected));
    }

    #[test]
    fn test_variable_string_not_regex() {
        let value = Value::from("hello");
        let variable = Variable::new(value.clone());
        let expected = Variable::new(Value::from("h.*z"));

        assert!(!variable.regex(&expected));
    }
}
