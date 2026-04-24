use crate::{ast::Value, error::{Result, DataMorphError}};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};

/// CSV adapter with sniffing, type inference, and intelligent handling
pub struct CsvAdapter {
    delimiter: u8,
    has_header: bool,
    quote: u8,
}

impl CsvAdapter {
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            has_header: true,
            quote: b'"',
        }
    }

    /// Probe a CSV file to detect delimiter, has_header, encoding
    pub fn probe(content: &str) -> CsvProbeResult {
        let mut result = CsvProbeResult::default();

        let lines: Vec<&str> = content.lines().take(10).collect();
        if lines.is_empty() {
            return result;
        }

        let delimiters = [b',', b'\t', b';', b'|'];
        let mut counts = [0; 4];

        for line in &lines {
            for (i, &delim) in delimiters.iter().enumerate() {
                counts[i] += line.bytes().filter(|&c| c == delim).count();
            }
        }

        let max_idx = counts.iter().enumerate().max_by_key(|(_, &c)| c).map(|(i, _)| i);
        if let Some(idx) = max_idx {
            if counts[idx] > 0 {
                result.delimiter = delimiters[idx];
                result.has_delimiter = true;
            }
        }

        if let Some(first_line) = lines.first() {
            let delim = if result.has_delimiter { result.delimiter } else { b',' };
            let mut reader = ReaderBuilder::new()
                .delimiter(delim)
                .has_headers(false)
                .from_reader(first_line.as_bytes());

            if let Some(Ok(record)) = reader.into_records().next() {
                result.has_header = Self::guess_header(&record);
            }
        }

        result
    }

    fn guess_header(record: &StringRecord) -> bool {
        let header_words = ["id", "name", "date", "created", "updated", "user", "email"];
        let mut header_score = 0;

        for (i, field) in record.iter().enumerate() {
            let field_lower = field.to_lowercase();
            if header_words.iter().any(|&word| field_lower.contains(word)) {
                header_score += 1;
            }
            if field.contains(' ') || (field.chars().next().unwrap().is_uppercase() && field.len() > 1) {
                header_score += 1;
            }
            if field.parse::<f64>().is_ok() && field.len() < 5 {
                header_score -= 1;
            }
        }

        header_score > 0
    }

    fn infer_type(value: &str) -> Value {
        if value.is_empty() {
            return Value::String(value.to_string());
        }

        if let Ok(i) = value.parse::<i64>() {
            return Value::Integer(i);
        }

        if let Ok(f) = value.parse::<f64>() {
            return Value::Float(f);
        }

        if let Ok(b) = value.parse::<bool>() {
            return Value::Bool(b);
        }

        Value::String(value.to_string())
    }

    pub fn parse(&self, input: &str) -> Result<Value> {
        let mut rdr = ReaderBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(self.has_header)
            .flexible(true)
            .from_reader(input.as_bytes());

        let mut records = Vec::new();

        let headers: Option<StringRecord> = if self.has_header {
            rdr.headers().map_err(|e| DataMorphError::ParseError {
                format: "CSV".to_string(),
                source: e.into(),
            })?.clone().into()
        } else {
            None
        };

        for result in rdr.records() {
            let record = result.map_err(|e| DataMorphError::ParseError {
                format: "CSV".to_string(),
                source: e.into(),
            })?;
            let mut obj = std::collections::BTreeMap::new();

            for (i, field) in record.iter().enumerate() {
                let key = if let Some(ref headers) = headers {
                    if i < headers.len() {
                        headers[i].to_string()
                    } else {
                        format!("column{}", i)
                    }
                } else {
                    format!("column{}", i)
                };

                let value = Self::infer_type(field);
                obj.insert(key, value);
            }

            records.push(Value::Object(obj));
        }

        Ok(Value::Array(records))
    }

    pub fn serialize(&self, value: &Value) -> Result<String> {
        let array = match value {
            Value::Array(arr) => arr,
            _ => return Err(DataMorphError::SerializeError {
                format: "CSV".to_string(),
                source: "CSV root must be an array".into(),
            }),
        };

        if array.is_empty() {
            return Ok(String::new());
        }

        let first_obj = match &array[0] {
            Value::Object(map) => map,
            _ => return Err(DataMorphError::SerializeError {
                format: "CSV".to_string(),
                source: "CSV array elements must be objects".into(),
            }),
        };

        let headers: Vec<String> = first_obj.keys().cloned().collect();

        let mut buf = Vec::new();
        {
            let mut wtr = WriterBuilder::new()
                .has_headers(true)
                .from_writer(&mut buf);

            for item in array {
                let obj = match item {
                    Value::Object(map) => map,
                    _ => return Err(DataMorphError::SerializeError {
                        format: "CSV".to_string(),
                        source: "CSV array elements must be objects".into(),
                    }),
                };

                let mut row = Vec::new();
                for key in &headers {
                    let val = obj.get(key).map(value_to_string).unwrap_or_default();
                    row.push(val);
                }

                wtr.write_record(&row).map_err(|e| DataMorphError::SerializeError {
                    format: "CSV".to_string(),
                    source: e.into(),
                })?;
            }
        }

        String::from_utf8(buf).map_err(|e| DataMorphError::SerializeError {
            format: "CSV".to_string(),
            source: e.into(),
        })
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) => "[]".to_string(),
        Value::Object(_) => "{}".to_string(),
    }
}

/// Result of CSV probing
#[derive(Debug, Clone, PartialEq)]
pub struct CsvProbeResult {
    pub delimiter: u8,
    pub has_delimiter: bool,
    pub has_header: bool,
}

impl Default for CsvProbeResult {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_delimiter: false,
            has_header: true,
        }
    }
}
