use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}
