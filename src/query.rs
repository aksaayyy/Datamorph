use crate::ast::Value;
use crate::error::DataMorphError;
use thiserror::Error;

/// Universal Path Language (UPL) parser and evaluator
///
/// Syntax examples:
///   `.users[0].name`           → access object field then array index
///   `users[*].name`            → map over array, extract name from each
///   `users[?age>30]`           → filter array by condition
///   `users[].{full: name, id}` → reshape objects
///   `users | map(.name)`       → pipe through function
///
/// This is a subset of jq-style syntax adapted for all data formats.

#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(usize),
    Wildcard,
    Filter(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Path(Vec<PathSegment>),
    Filter {
        input: Box<Expression>,
        predicate: Box<Expression>,
    },
    Map {
        input: Box<Expression>,
        transform: Box<Expression>,
    },
    Literal(Value),
    Field(String),  // Field access in current context
    Variable(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Eq, Ne, Gt, Lt, Gte, LTE,
    Add, Sub, Mul, Div,
    And, Or,
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: &'static str, got: &'static str },

    #[error("Filter error: {0}")]
    FilterError(String),
}

pub struct UplEvaluator;

impl UplEvaluator {
    /// Evaluate a UPL expression against a Value
    pub fn evaluate(expr: &Expression, value: &Value) -> Result<Value, QueryError> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            Expression::Variable(name) => Err(QueryError::InvalidPath(format!(
                "Variables not supported yet: {}", name
            ))),
            Expression::Field(name) => {
                match value {
                    Value::Object(map) => map.get(name)
                        .cloned()
                        .ok_or_else(|| QueryError::KeyNotFound(name.clone())),
                    _ => Err(QueryError::TypeError {
                        expected: "object",
                        got: "array or scalar",
                    }),
                }
            }
            Expression::Path(segments) => {
                let mut current = value.clone();
                for segment in segments {
                    current = Self::apply_segment(&current, segment)?;
                }
                Ok(current.clone())
            }
            Expression::Map { input, transform } => {
                let arr = Self::evaluate(input, value)?;
                match arr {
                    Value::Array(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            let transformed = Self::evaluate(transform, &item)?;
                            result.push(transformed);
                        }
                        Ok(Value::Array(result))
                    }
                    _ => Err(QueryError::TypeError {
                        expected: "array",
                        got: "value",
                    }),
                }
            }
            Expression::Filter { input, predicate } => {
                let arr = Self::evaluate(input, value)?;
                match arr {
                    Value::Array(items) => {
                        let mut result = Vec::new();
                        for item in &items {
                            let predicate_val = Self::evaluate(predicate, item)?;
                            if Self::is_truthy(&predicate_val) {
                                result.push(item.clone());
                            }
                        }
                        Ok(Value::Array(result))
                    }
                    _ => Err(QueryError::TypeError {
                        expected: "array",
                        got: "value",
                    }),
                }
            }
            Expression::BinaryOp { left, op, right } => {
                let lval = Self::evaluate(left, value)?;
                let rval = Self::evaluate(right, value)?;
                Self::eval_binary_op(lval, *op, rval)
            }
        }
    }

    fn apply_segment(value: &Value, segment: &PathSegment) -> Result<Value, QueryError> {
        match segment {
            PathSegment::Field(name) => {
                match value {
                    Value::Object(map) => map.get(name)
                        .cloned()
                        .ok_or_else(|| QueryError::KeyNotFound(name.clone())),
                    Value::Array(arr) => {
                        // Map over array: extract field from each object element
                        let mut results = Vec::new();
                        for item in arr {
                            match item {
                                Value::Object(map) => {
                                    let val = map.get(name).cloned().unwrap_or(Value::Null);
                                    results.push(val);
                                }
                                _ => results.push(Value::Null),
                            }
                        }
                        Ok(Value::Array(results))
                    }
                    _ => Err(QueryError::TypeError {
                        expected: "object",
                        got: "array or scalar",
                    }),
                }
            }
            PathSegment::Index(idx) => {
                match value {
                    Value::Array(arr) => arr.get(*idx)
                        .cloned()
                        .ok_or_else(|| QueryError::IndexOutOfBounds(*idx)),
                    _ => Err(QueryError::TypeError {
                        expected: "array",
                        got: "object or scalar",
                    }),
                }
            }
            PathSegment::Wildcard => {
                match value {
                    Value::Array(arr) => Ok(Value::Array(arr.clone())),
                    Value::Object(map) => {
                        let values: Vec<Value> = map.values().cloned().collect();
                        Ok(Value::Array(values))
                    }
                    _ => Ok(value.clone()),
                }
            }
            PathSegment::Filter(expr) => {
                // Filter expects an array; if given object, treat as array of values
                let arr = match value {
                    Value::Array(a) => a.clone(),
                    Value::Object(map) => map.values().cloned().collect(),
                    _ => return Err(QueryError::TypeError {
                        expected: "array or object",
                        got: "scalar",
                    }),
                };
                let mut result = Vec::new();
                for item in &arr {
                    let predicate_val = Self::evaluate(expr, item)?;
                    if Self::is_truthy(&predicate_val) {
                        result.push(item.clone());
                    }
                }
                Ok(Value::Array(result))
            }
        }
    }

    fn eval_binary_op(
        left: Value,
        op: BinaryOperator,
        right: Value,
    ) -> Result<Value, QueryError> {
        use BinaryOperator::*;
        match (op, &left, &right) {
            (Add, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Add, Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Sub, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Mul, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Div, Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(QueryError::FilterError("division by zero".to_string()))
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Eq, a, b) => Ok(Value::Bool(a == b)),
            (Ne, a, b) => Ok(Value::Bool(a != b)),
            (Gt, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
            (Lt, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
            (Gte, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
            (LTE, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(QueryError::TypeError {
                expected: "compatible numeric or string types",
                got: "mismatched",
            }),
        }
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(map) => !map.is_empty(),
        }
    }
}

/// Parse a simple UPL path string into Expression
/// Supports: field.field, field[index], field[*], field[?cond]
pub fn parse_upl(path: &str) -> Result<Expression, QueryError> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Field(current.clone()));
                    current.clear();
                }
                // Skip consecutive dots
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Field(current.clone()));
                    current.clear();
                }
                // Parse bracket content until closing ']'
                let mut bracket_content = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(c) => bracket_content.push(c),
                        None => return Err(QueryError::InvalidPath("Unclosed bracket".to_string())),
                    }
                }
                let trimmed = bracket_content.trim();
                if trimmed == "*" {
                    segments.push(PathSegment::Wildcard);
                } else if let Ok(idx) = trimmed.parse::<usize>() {
                    segments.push(PathSegment::Index(idx));
                } else if trimmed.starts_with("?") {
                    let filter_expr = parse_filter_expression(&trimmed[1..])?;
                    segments.push(PathSegment::Filter(filter_expr));
                } else {
                    return Err(QueryError::InvalidPath(
                        format!("Invalid bracket content: {}", trimmed)
                    ));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        segments.push(PathSegment::Field(current));
    }

    Ok(Expression::Path(segments))
}

fn parse_filter_condition(expr: &str) -> Result<Expression, QueryError> {
    // Very simple: field>number or field=="string"
    // For now, support only simple binary comparisons
    let operators = ["==", "!=", ">", "<", ">=", "<="];
    for op_str in &operators {
        if let Some(idx) = expr.find(op_str) {
            let left = expr[..idx].trim();
            let right = expr[idx + op_str.len()..].trim();

            let op = match *op_str {
                "==" => BinaryOperator::Eq,
                "!=" => BinaryOperator::Ne,
                ">" => BinaryOperator::Gt,
                "<" => BinaryOperator::Lt,
                ">=" => BinaryOperator::Gte,
                "<=" => BinaryOperator::LTE,
                _ => unreachable!(),
            };

            let lhs = Expression::Field(left.to_string());
            let rhs = if let Ok(int_val) = right.parse::<i64>() {
                Expression::Literal(Value::Integer(int_val))
            } else if let Ok(float_val) = right.parse::<f64>() {
                Expression::Literal(Value::Float(float_val))
            } else if right.starts_with('"') && right.ends_with('"') {
                Expression::Literal(Value::String(right[1..right.len()-1].to_string()))
            } else if right == "true" {
                Expression::Literal(Value::Bool(true))
            } else if right == "false" {
                Expression::Literal(Value::Bool(false))
            } else {
                return Err(QueryError::FilterError(
                    format!("Unsupported filter value: {}", right)
                ));
            };

            return Ok(Expression::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            });
        }
    }

    Err(QueryError::InvalidPath(
        format!("Cannot parse filter condition: {}", expr)
    ))
}

fn parse_filter_expression(expr: &str) -> Result<Expression, QueryError> {
    // For now, filter is just a condition that evaluates to bool
    parse_filter_condition(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Value;

    #[test]
    fn test_parse_simple_path() {
        let expr = parse_upl("users.name").unwrap();
        match expr {
            Expression::Path(segments) => {
                assert_eq!(segments.len(), 2);
                match &segments[0] {
                    PathSegment::Field(f) => assert_eq!(f, "users"),
                    _ => panic!("Expected Field"),
                }
                match &segments[1] {
                    PathSegment::Field(f) => assert_eq!(f, "name"),
                    _ => panic!("Expected Field"),
                }
            }
            _ => panic!("Expected Path"),
        }
    }

    #[test]
    fn test_parse_array_index() {
        let expr = parse_upl("items[0]").unwrap();
        match expr {
            Expression::Path(segments) => {
                assert_eq!(segments.len(), 2);
                match &segments[1] {
                    PathSegment::Index(idx) => assert_eq!(*idx, 0),
                    _ => panic!("Expected Index"),
                }
            }
            _ => panic!("Expected Path"),
        }
    }

    #[test]
    fn test_parse_wildcard() {
        let expr = parse_upl("users[*]").unwrap();
        match expr {
            Expression::Path(segments) => {
                assert_eq!(segments.len(), 2);
                assert!(matches!(segments[1], PathSegment::Wildcard));
            }
            _ => panic!("Expected Path"),
        }
    }
}