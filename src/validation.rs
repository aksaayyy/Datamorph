use crate::{ast::Value, error::{DataMorphError, Result}};
use jsonschema::{JSONSchema, Draft};
use serde_json::Value as JsonValue;
use std::path::Path;

/// Validates a JSON value against a JSON Schema file
pub struct SchemaValidator {
    schema: JSONSchema,
}

impl SchemaValidator {
    /// Load and compile a JSON Schema from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let schema_content = std::fs::read_to_string(path)
            .map_err(|e| DataMorphError::IoError(e))?;

        let schema_json: JsonValue = serde_json::from_str(&schema_content)
            .map_err(|e| DataMorphError::ParseError {
                format: "JSON Schema".to_string(),
                source: Box::new(e),
            })?;

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_json)
            .map_err(|e| DataMorphError::SchemaValidationError(e.to_string()))?;

        Ok(Self { schema: compiled })
    }

    /// Load schema from a JSON string (for testing)
    pub fn from_str(schema_str: &str) -> Result<Self> {
        let schema_json: JsonValue = serde_json::from_str(schema_str)
            .map_err(|e| DataMorphError::ParseError {
                format: "JSON Schema".to_string(),
                source: Box::new(e),
            })?;

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_json)
            .map_err(|e| DataMorphError::SchemaValidationError(e.to_string()))?;

        Ok(Self { schema: compiled })
    }

    /// Validate a JSON value against the loaded schema
    /// Returns Ok(()) if valid, or Err with detailed error messages
    pub fn validate(&self, instance: &JsonValue) -> Result<()> {
        self.schema
            .validate(instance)
            .map_err(|errors| {
                let mut msg = String::from("Schema validation failed:\n");
                for error in errors {
                    // Use Display which includes path and message
                    msg.push_str(&format!("  • {}\n", error));
                }
                DataMorphError::SchemaValidationError(msg.trim().to_string())
            })?;
        Ok(())
    }

    /// Validate our internal AST Value (converts to serde_json::Value first)
    pub fn validate_ast(&self, value: &Value) -> Result<()> {
        let json_value = ast_to_serde_json(value);
        self.validate(&json_value)
    }
}

/// Convert Datamorph's Value AST to serde_json::Value for schema validation
fn ast_to_serde_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Integer(i) => JsonValue::Number((*i).into()),
        Value::Float(f) => {
            if let Some(num) = serde_json::Number::from_f64(*f) {
                JsonValue::Number(num)
            } else {
                JsonValue::Null
            }
        }
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Array(arr) => {
            JsonValue::Array(arr.iter().map(|v| ast_to_serde_json(v)).collect())
        }
        Value::Object(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), ast_to_serde_json(v));
            }
            JsonValue::Object(obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validation_valid() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        });

        let validator = SchemaValidator::from_str(&serde_json::to_string(&schema).unwrap()).unwrap();

        let instance = json!({
            "name": "Alice",
            "age": 30
        });

        assert!(validator.validate(&instance).is_ok());
    }

    #[test]
    fn test_schema_validation_invalid() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": { "type": "integer", "minimum": 0 }
            }
        });

        let validator = SchemaValidator::from_str(&serde_json::to_string(&schema).unwrap()).unwrap();

        let instance = json!({
            "age": -5
        });

        let result = validator.validate(&instance);
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Schema validation failed"));
        assert!(err_msg.contains("minimum"));
    }

    #[test]
    fn test_schema_missing_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });

        let validator = SchemaValidator::from_str(&serde_json::to_string(&schema).unwrap()).unwrap();

        let instance = json!({});  // missing required field

        let result = validator.validate(&instance);
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("required"));
    }
}
