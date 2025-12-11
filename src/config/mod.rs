use crate::models::{ConnectionConfig, DatabaseType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),
    #[error("Config directory not found")]
    ConfigDirNotFound,
}

/// Raw connection config as stored in TOML (without runtime fields like UUID)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredConnection {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub ssl_enabled: bool,
    #[serde(default)]
    pub color: Option<String>,
    // For SQLite
    #[serde(default)]
    pub file: Option<String>,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NebulaSettings {
    #[serde(default)]
    pub last_connection: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
}

/// Root configuration structure matching config.toml format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub mysql: HashMap<String, StoredConnection>,
    #[serde(default)]
    pub postgres: HashMap<String, StoredConnection>,
    #[serde(default)]
    pub sqlite: HashMap<String, StoredConnection>,
    #[serde(default)]
    pub mongodb: HashMap<String, StoredConnection>,
    #[serde(default)]
    pub nebula: NebulaSettings,
}

impl AppConfig {
    /// Get the config directory path (~/.config/nebula on all platforms)
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        dirs::home_dir()
            .map(|p| p.join(".config").join("nebula"))
            .ok_or(ConfigError::ConfigDirNotFound)
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)?;

        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Convert stored connections to runtime ConnectionConfig objects
    pub fn get_connections(&self) -> Vec<ConnectionConfig> {
        let mut connections = Vec::new();

        // MySQL connections
        for (key, stored) in &self.mysql {
            connections.push(stored_to_connection_config(
                key,
                stored,
                DatabaseType::MySQL,
            ));
        }

        // PostgreSQL connections
        for (key, stored) in &self.postgres {
            connections.push(stored_to_connection_config(
                key,
                stored,
                DatabaseType::PostgreSQL,
            ));
        }

        // SQLite connections
        for (key, stored) in &self.sqlite {
            let mut conn = stored_to_connection_config(key, stored, DatabaseType::SQLite);
            // For SQLite, use file path as database
            if let Some(file) = &stored.file {
                conn.database = file.clone();
            }
            connections.push(conn);
        }

        // MongoDB connections
        for (key, stored) in &self.mongodb {
            connections.push(stored_to_connection_config(
                key,
                stored,
                DatabaseType::MongoDB,
            ));
        }

        connections
    }

    /// Add or update a connection
    pub fn save_connection(&mut self, config: &ConnectionConfig) {
        let key = config.name.clone();
        let stored = connection_config_to_stored(config);

        match config.db_type {
            DatabaseType::MySQL => {
                self.mysql.insert(key, stored);
            }
            DatabaseType::PostgreSQL => {
                self.postgres.insert(key, stored);
            }
            DatabaseType::SQLite => {
                let mut stored = stored;
                stored.file = Some(config.database.clone());
                self.sqlite.insert(key, stored);
            }
            DatabaseType::MongoDB => {
                self.mongodb.insert(key, stored);
            }
        }
    }

    /// Remove a connection by name and type
    pub fn remove_connection(&mut self, name: &str, db_type: DatabaseType) {
        match db_type {
            DatabaseType::MySQL => {
                self.mysql.remove(name);
            }
            DatabaseType::PostgreSQL => {
                self.postgres.remove(name);
            }
            DatabaseType::SQLite => {
                self.sqlite.remove(name);
            }
            DatabaseType::MongoDB => {
                self.mongodb.remove(name);
            }
        }
    }

    /// Set the last used connection
    pub fn set_last_connection(&mut self, name: &str) {
        self.nebula.last_connection = Some(name.to_string());
    }
}

fn stored_to_connection_config(
    key: &str,
    stored: &StoredConnection,
    db_type: DatabaseType,
) -> ConnectionConfig {
    let name = if stored.name.is_empty() {
        key.to_string()
    } else {
        stored.name.clone()
    };

    ConnectionConfig {
        id: Uuid::new_v4(),
        name,
        db_type,
        host: stored.host.clone(),
        port: stored.port.unwrap_or(db_type.default_port()),
        username: stored.username.clone(),
        password: stored.password.clone(),
        database: stored.database.clone(),
        ssl_enabled: stored.ssl_enabled,
        color: stored.color.clone(),
    }
}

fn connection_config_to_stored(config: &ConnectionConfig) -> StoredConnection {
    StoredConnection {
        name: config.name.clone(),
        host: config.host.clone(),
        port: Some(config.port),
        username: config.username.clone(),
        password: config.password.clone(),
        database: config.database.clone(),
        ssl_enabled: config.ssl_enabled,
        color: config.color.clone(),
        file: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.mysql.is_empty());
        assert!(config.postgres.is_empty());
        assert!(config.nebula.last_connection.is_none());
    }

    #[test]
    fn test_parse_example_config() {
        let toml_content = r#"
[mysql.default]
host = "localhost"
port = 3306
username = "root"
password = "secret"
database = "mydb"

[postgres.production]
host = "db.example.com"
port = 5432
username = "admin"
password = "secure"
database = "app"

[sqlite.local]
file = "/path/to/database.db"

[nebula]
last_connection = "mysql.default"
"#;

        let config: AppConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.mysql.len(), 1);
        assert_eq!(config.postgres.len(), 1);
        assert_eq!(config.sqlite.len(), 1);
        assert_eq!(
            config.nebula.last_connection,
            Some("mysql.default".to_string())
        );
    }
}
