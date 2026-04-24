use crate::ast::Value;
use crate::error::DataMorphError;
use crate::csv_adapter::CsvAdapter;
use serde_json;
use serde_yaml;
use toml;

/// Adapter enum — object-safe, no trait objects needed
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Adapter {
    Json,
    Yaml,
    Toml,
    Csv,
}

impl Adapter {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Csv => "CSV",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Json => &["json"],
            Self::Yaml => &["yaml", "yml"],
            Self::Toml => &["toml"],
            Self::Csv => &["csv"],
        }
    }

    pub fn parse(&self, input: &str) -> Result<Value, DataMorphError> {
        match self {
            Self::Json => serde_json::from_str(input).map_err(|e| DataMorphError::ParseError {
                format: "JSON".to_string(),
                source: e.into(),
            }),
            Self::Yaml => serde_yaml::from_str(input).map_err(|e| DataMorphError::ParseError {
                format: "YAML".to_string(),
                source: e.into(),
            }),
            Self::Toml => toml::from_str(input).map_err(|e| DataMorphError::ParseError {
                format: "TOML".to_string(),
                source: e.into(),
            }),
            Self::Csv => CsvAdapter::new().parse(input),
        }
    }

    pub fn serialize(&self, value: &Value) -> Result<String, DataMorphError> {
        match self {
            Self::Json => serde_json::to_string_pretty(value).map_err(|e| DataMorphError::SerializeError {
                format: "JSON".to_string(),
                source: e.into(),
            }),
            Self::Yaml => serde_yaml::to_string(value).map_err(|e| DataMorphError::SerializeError {
                format: "YAML".to_string(),
                source: e.into(),
            }),
            Self::Toml => toml::to_string_pretty(value).map_err(|e| DataMorphError::SerializeError {
                format: "TOML".to_string(),
                source: e.into(),
            }),
            Self::Csv => CsvAdapter::new().serialize(value),
        }
    }
}

/// Get adapter by format name
pub fn get_adapter_by_name(name: &str) -> Option<Adapter> {
    match name.to_lowercase().as_str() {
        "json" => Some(Adapter::Json),
        "yaml" | "yml" => Some(Adapter::Yaml),
        "toml" => Some(Adapter::Toml),
        "csv" => Some(Adapter::Csv),
        _ => None,
    }
}

/// Get adapter by file extension
pub fn get_adapter_by_extension(ext: &str) -> Option<Adapter> {
    let ext = ext.to_lowercase();
    match ext.as_str() {
        "json" => Some(Adapter::Json),
        "yaml" | "yml" => Some(Adapter::Yaml),
        "toml" => Some(Adapter::Toml),
        "csv" => Some(Adapter::Csv),
        _ => None,
    }
}