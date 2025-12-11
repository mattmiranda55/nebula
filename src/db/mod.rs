pub mod mysql;
// pub mod postgres; // TODO: Implement PostgreSQL support
// pub mod sqlite;   // TODO: Implement SQLite support
// pub mod mongodb;  // TODO: Implement MongoDB support

use crate::models::{ConnectionConfig, DatabaseType, QueryResult};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Database not found: {0}")]
    DatabaseNotFound(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Unsupported database type: {0}")]
    UnsupportedType(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Schema object types
#[derive(Debug, Clone)]
pub enum SchemaObject {
    Database(DatabaseInfo),
    Table(TableInfo),
    View(ViewInfo),
    Column(ColumnDetails),
    Index(IndexInfo),
}

#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub name: String,
    pub character_set: Option<String>,
    pub collation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub database: String,
    pub engine: Option<String>,
    pub row_count: Option<u64>,
    pub data_size: Option<u64>,
    pub columns: Vec<ColumnDetails>,
}

#[derive(Debug, Clone)]
pub struct ViewInfo {
    pub name: String,
    pub database: String,
    pub definition: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnDetails {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_auto_increment: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}

/// Database connection trait
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Test the connection
    async fn test_connection(&self) -> Result<(), DatabaseError>;

    /// Get list of databases
    async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, DatabaseError>;

    /// Get tables in a database
    async fn list_tables(&self, database: &str) -> Result<Vec<TableInfo>, DatabaseError>;

    /// Get views in a database
    async fn list_views(&self, database: &str) -> Result<Vec<ViewInfo>, DatabaseError>;

    /// Get table structure
    async fn describe_table(&self, database: &str, table: &str) -> Result<TableInfo, DatabaseError>;

    /// Execute a query and return results
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DatabaseError>;

    /// Execute a query without returning results (INSERT, UPDATE, DELETE)
    async fn execute_statement(&self, sql: &str) -> Result<u64, DatabaseError>;

    /// Get table data with pagination
    async fn get_table_data(
        &self,
        database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DatabaseError>;

    /// Close the connection
    async fn close(&self) -> Result<(), DatabaseError>;
}

/// Create a database connection based on config
pub async fn create_connection(
    config: &ConnectionConfig,
) -> Result<Box<dyn DatabaseConnection>, DatabaseError> {
    match config.db_type {
        DatabaseType::MySQL => {
            let conn = mysql::MySqlConnection::connect(config).await?;
            Ok(Box::new(conn))
        }
        DatabaseType::PostgreSQL => {
            // TODO: Implement PostgreSQL
            Err(DatabaseError::UnsupportedType(
                "PostgreSQL support coming soon".to_string(),
            ))
        }
        DatabaseType::SQLite => {
            // TODO: Implement SQLite
            Err(DatabaseError::UnsupportedType(
                "SQLite support coming soon".to_string(),
            ))
        }
        DatabaseType::MongoDB => {
            // TODO: Implement MongoDB
            Err(DatabaseError::UnsupportedType(
                "MongoDB support coming soon".to_string(),
            ))
        }
    }
}
