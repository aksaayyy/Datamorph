use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataMorphError {
    #[error("Parse error for format {format}: {source}")]
    ParseError {
        format: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Serialization error for format {format}: {source}")]
    SerializeError {
        format: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Format detection failed for file: {0}")]
    FormatDetectionFailed(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Diff error: {0}")]
    DiffError(String),

    #[error("Lint error: {0}")]
    LintError(String),

    #[error("Repair error: {0}")]
    RepairError(String),
}

pub type Result<T> = std::result::Result<T, DataMorphError>;
