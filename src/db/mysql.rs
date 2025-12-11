use crate::db::{
    ColumnDetails, DatabaseConnection, DatabaseError, DatabaseInfo, TableInfo, ViewInfo,
};
use crate::models::{CellValue, ColumnInfo, ConnectionConfig, QueryResult};
use async_trait::async_trait;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row, TypeInfo};
use std::time::Instant;

pub struct MySqlConnection {
    pool: MySqlPool,
}

impl MySqlConnection {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, DatabaseError> {
        let url = config.connection_string();

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    fn row_to_values(row: &MySqlRow) -> Vec<CellValue> {
        let mut values = Vec::new();
        for i in 0..row.len() {
            let col = row.column(i);
            let type_name = col.type_info().name();

            let value = match type_name {
                "BOOLEAN" | "BOOL" => row
                    .try_get::<bool, _>(i)
                    .map(CellValue::Bool)
                    .unwrap_or(CellValue::Null),
                "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => row
                    .try_get::<i64, _>(i)
                    .map(CellValue::Int)
                    .unwrap_or(CellValue::Null),
                "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED"
                | "MEDIUMINT UNSIGNED" | "BIGINT UNSIGNED" => row
                    .try_get::<i64, _>(i)
                    .map(CellValue::Int)
                    .unwrap_or(CellValue::Null),
                "FLOAT" | "DOUBLE" | "DECIMAL" => row
                    .try_get::<f64, _>(i)
                    .map(CellValue::Float)
                    .unwrap_or(CellValue::Null),
                "DATE" | "TIME" | "DATETIME" | "TIMESTAMP" => row
                    .try_get::<String, _>(i)
                    .map(CellValue::DateTime)
                    .unwrap_or(CellValue::Null),
                "JSON" => row
                    .try_get::<String, _>(i)
                    .map(CellValue::Json)
                    .unwrap_or(CellValue::Null),
                "BLOB" | "BINARY" | "VARBINARY" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" => row
                    .try_get::<Vec<u8>, _>(i)
                    .map(CellValue::Bytes)
                    .unwrap_or(CellValue::Null),
                _ => row
                    .try_get::<String, _>(i)
                    .map(CellValue::String)
                    .unwrap_or(CellValue::Null),
            };
            values.push(value);
        }
        values
    }
}

#[async_trait]
impl DatabaseConnection for MySqlConnection {
    async fn test_connection(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }

    async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, DatabaseError> {
        let rows: Vec<MySqlRow> = sqlx::query("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let databases = rows
            .iter()
            .filter_map(|row| {
                row.try_get::<String, _>(0).ok().map(|name| DatabaseInfo {
                    name,
                    character_set: None,
                    collation: None,
                })
            })
            .collect();

        Ok(databases)
    }

    async fn list_tables(&self, database: &str) -> Result<Vec<TableInfo>, DatabaseError> {
        let query = format!(
            "SELECT TABLE_NAME, ENGINE, TABLE_ROWS, DATA_LENGTH 
             FROM information_schema.TABLES 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_TYPE = 'BASE TABLE'",
            database
        );

        let rows: Vec<MySqlRow> = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let tables = rows
            .iter()
            .filter_map(|row| {
                let name: String = row.try_get(0).ok()?;
                Some(TableInfo {
                    name,
                    database: database.to_string(),
                    engine: row.try_get(1).ok(),
                    row_count: row.try_get::<i64, _>(2).ok().map(|v| v as u64),
                    data_size: row.try_get::<i64, _>(3).ok().map(|v| v as u64),
                    columns: Vec::new(),
                })
            })
            .collect();

        Ok(tables)
    }

    async fn list_views(&self, database: &str) -> Result<Vec<ViewInfo>, DatabaseError> {
        let query = format!(
            "SELECT TABLE_NAME, VIEW_DEFINITION 
             FROM information_schema.VIEWS 
             WHERE TABLE_SCHEMA = '{}'",
            database
        );

        let rows: Vec<MySqlRow> = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let views = rows
            .iter()
            .filter_map(|row| {
                let name: String = row.try_get(0).ok()?;
                Some(ViewInfo {
                    name,
                    database: database.to_string(),
                    definition: row.try_get(1).ok(),
                })
            })
            .collect();

        Ok(views)
    }

    async fn describe_table(&self, database: &str, table: &str) -> Result<TableInfo, DatabaseError> {
        let query = format!(
            "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, 
                    COLUMN_KEY, EXTRA, COLUMN_COMMENT
             FROM information_schema.COLUMNS 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}'
             ORDER BY ORDINAL_POSITION",
            database, table
        );

        let rows: Vec<MySqlRow> = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let columns = rows
            .iter()
            .filter_map(|row| {
                let name: String = row.try_get(0).ok()?;
                let data_type: String = row.try_get(1).ok()?;
                let nullable: String = row.try_get(2).ok()?;
                let column_key: String = row.try_get::<String, _>(4).unwrap_or_default();
                let extra: String = row.try_get::<String, _>(5).unwrap_or_default();

                Some(ColumnDetails {
                    name,
                    data_type,
                    nullable: nullable == "YES",
                    default_value: row.try_get(3).ok(),
                    is_primary_key: column_key == "PRI",
                    is_auto_increment: extra.contains("auto_increment"),
                    comment: row.try_get(6).ok(),
                })
            })
            .collect();

        Ok(TableInfo {
            name: table.to_string(),
            database: database.to_string(),
            engine: None,
            row_count: None,
            data_size: None,
            columns,
        })
    }

    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DatabaseError> {
        let start = Instant::now();

        let rows: Vec<MySqlRow> = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                affected_rows: None,
                execution_time_ms,
            });
        }

        // Extract column info from first row
        let columns: Vec<ColumnInfo> = rows[0]
            .columns()
            .iter()
            .map(|col| ColumnInfo {
                name: col.name().to_string(),
                data_type: col.type_info().name().to_string(),
                nullable: true,
                is_primary_key: false,
            })
            .collect();

        let data_rows: Vec<Vec<CellValue>> =
            rows.iter().map(|row| Self::row_to_values(row)).collect();

        Ok(QueryResult {
            columns,
            rows: data_rows,
            affected_rows: None,
            execution_time_ms,
        })
    }

    async fn execute_statement(&self, sql: &str) -> Result<u64, DatabaseError> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn get_table_data(
        &self,
        database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult, DatabaseError> {
        let sql = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
            database, table, limit, offset
        );
        self.execute_query(&sql).await
    }

    async fn close(&self) -> Result<(), DatabaseError> {
        self.pool.close().await;
        Ok(())
    }
}
