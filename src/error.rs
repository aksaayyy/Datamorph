    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Schema validation error: {0}")]
    SchemaValidationError(String),

    #[error("Function error: {0}")]
    FunctionError(String),

    #[error("Diff error: {0}")]
    DiffError(String),