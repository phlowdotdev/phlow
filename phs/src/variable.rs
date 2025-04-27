use regex::Regex;
use valu3::{prelude::*, value::Value};

#[derive(Debug, PartialEq)]
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
        match (self, other) {
            (
                Variable {
                    value: Value::Number(left),
                    ..
                },
                Variable {
                    value: Value::Number(right),
                    ..
                },
            ) => {
                if left.is_float() {
                    left.to_f64() == right.to_f64()
                } else {
                    left.to_i64() == right.to_i64()
                }
            }
            _ => self.get() == other.get(),
        }
    }

    pub fn greater_than(&self, other: &Variable) -> bool {
        match (self, other) {
            (
                Variable {
                    value: Value::Number(left),
                    ..
                },
                Variable {
                    value: Value::Number(right),
                    ..
                },
            ) => {
                if left.is_float() {
                    left.to_f64() > right.to_f64()
                } else {
                    left.to_i64() > right.to_i64()
                }
            }
            _ => self.get() > other.get(),
        }
    }

    pub fn less_than(&self, other: &Variable) -> bool {
        match (self, other) {
            (
                Variable {
                    value: Value::Number(left),
                    ..
                },
                Variable {
                    value: Value::Number(right),
                    ..
                },
            ) => {
                if left.is_float() {
                    left.to_f64() < right.to_f64()
                } else {
                    left.to_i64() < right.to_i64()
                }
            }
            _ => self.get() < other.get(),
        }
    }

    pub fn greater_than_or_equal(&self, other: &Variable) -> bool {
        match (self, other) {
            (
                Variable {
                    value: Value::Number(left),
                    ..
                },
                Variable {
                    value: Value::Number(right),
                    ..
                },
            ) => {
                if left.is_float() {
                    left.to_f64() <= right.to_f64()
                } else {
                    left.to_i64() <= right.to_i64()
                }
            }
            _ => self.get() < other.get(),
        }
    }

    pub fn less_than_or_equal(&self, other: &Variable) -> bool {
        match (self, other) {
            (
                Variable {
                    value: Value::Number(left),
                    ..
                },
                Variable {
                    value: Value::Number(right),
                    ..
                },
            ) => {
                if left.is_float() {
                    left.to_f64() >= right.to_f64()
                } else {
                    left.to_i64() >= right.to_i64()
                }
            }
            _ => self.get() < other.get(),
        }
    }

    pub fn contains(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let target = self.get().as_str();
            let other = other.get().as_str();

            return target.contains(other);
        } else if self.get().is_array() && other.get().is_array() {
            let target = match self.get().as_array() {
                Some(array) => array,
                None => return false,
            };
            let other = match other.get().as_array() {
                Some(array) => array,
                None => return false,
            };

            return target
                .into_iter()
                .any(|x| other.into_iter().any(|y| x == y));
        }

        return false;
    }

    pub fn starts_with(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let target = self.get().as_str();
            let other = other.get().as_str();

            return target.starts_with(other);
        }

        return false;
    }

    pub fn ends_with(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let target = self.get().as_str();
            let other = other.get().as_str();

            return target.ends_with(other);
        }

        return false;
    }

    pub fn regex(&self, other: &Variable) -> bool {
        if self.get().is_string() && other.get().is_string() {
            let target = self.get().as_str();
            let other = other.get().as_str();

            return match Regex::new(other) {
                Ok(re) => re.is_match(target),
                Err(_) => false,
            };
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
