use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How to display boolean values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoolDisplayFormat {
    #[default]
    Checkbox,
    TrueFalse,
    OneZero,
}

impl BoolDisplayFormat {
    pub fn display(&self, value: bool) -> String {
        match self {
            BoolDisplayFormat::Checkbox => if value { "☑" } else { "☐" }.to_string(),
            BoolDisplayFormat::TrueFalse => if value { "true" } else { "false" }.to_string(),
            BoolDisplayFormat::OneZero => if value { "1" } else { "0" }.to_string(),
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            BoolDisplayFormat::Checkbox => "Checkbox",
            BoolDisplayFormat::TrueFalse => "true/false",
            BoolDisplayFormat::OneZero => "1/0",
        }
    }
}

/// A saved query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub id: Uuid,
    pub name: String,
    pub sql: String,
    pub connection_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for SavedQuery {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: "Untitled Query".to_string(),
            sql: String::new(),
            connection_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl SavedQuery {
    pub fn new(name: String, sql: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            sql,
            connection_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Query result data
#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<CellValue>>,
    pub affected_rows: Option<u64>,
    pub execution_time_ms: u64,
}

/// Column information
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

/// Cell value in query results
#[derive(Debug, Clone)]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    DateTime(String),
    Json(String),
}

impl std::fmt::Display for CellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellValue::Null => write!(f, "NULL"),
            CellValue::Bool(b) => write!(f, "{}", b),
            CellValue::Int(i) => write!(f, "{}", i),
            CellValue::Float(fl) => write!(f, "{}", fl),
            CellValue::String(s) => write!(f, "{}", s),
            CellValue::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            CellValue::DateTime(dt) => write!(f, "{}", dt),
            CellValue::Json(j) => write!(f, "{}", j),
        }
    }
}

impl CellValue {
    pub fn display_string(&self) -> String {
        self.to_string()
    }
    
    pub fn display_with_format(&self, bool_format: BoolDisplayFormat) -> String {
        match self {
            CellValue::Bool(b) => bool_format.display(*b),
            _ => self.to_string(),
        }
    }
    
    /// Convert to SQL literal for use in UPDATE statements
    pub fn to_sql_literal(&self) -> String {
        match self {
            CellValue::Null => "NULL".to_string(),
            CellValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            CellValue::Int(i) => i.to_string(),
            CellValue::Float(f) => f.to_string(),
            CellValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            CellValue::Bytes(_) => "NULL".to_string(), // Can't easily edit bytes
            CellValue::DateTime(dt) => format!("'{}'", dt),
            CellValue::Json(j) => format!("'{}'", j.replace('\'', "''")),
        }
    }
    
    /// Parse a string into a CellValue based on the original type
    pub fn parse_from_string(s: &str, original: &CellValue) -> CellValue {
        if s.eq_ignore_ascii_case("null") {
            return CellValue::Null;
        }
        
        match original {
            CellValue::Null => {
                // Try to infer type from string
                if let Ok(i) = s.parse::<i64>() {
                    CellValue::Int(i)
                } else if let Ok(f) = s.parse::<f64>() {
                    CellValue::Float(f)
                } else if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
                    CellValue::Bool(s.eq_ignore_ascii_case("true"))
                } else {
                    CellValue::String(s.to_string())
                }
            }
            CellValue::Bool(_) => {
                CellValue::Bool(s.eq_ignore_ascii_case("true") || s == "1")
            }
            CellValue::Int(_) => {
                s.parse::<i64>().map(CellValue::Int).unwrap_or(CellValue::String(s.to_string()))
            }
            CellValue::Float(_) => {
                s.parse::<f64>().map(CellValue::Float).unwrap_or(CellValue::String(s.to_string()))
            }
            CellValue::String(_) => CellValue::String(s.to_string()),
            CellValue::Bytes(_) => CellValue::String(s.to_string()),
            CellValue::DateTime(_) => CellValue::DateTime(s.to_string()),
            CellValue::Json(_) => CellValue::Json(s.to_string()),
        }
    }
}
