use std::collections::HashMap;

use serde::Deserialize;

use super::split_env_field;
use super::Validate;
use crate::error::{ConfigResult, Error};

// ──────────────────────────────────────────────
// MySQL 配置
// ──────────────────────────────────────────────

/// 单个 MySQL 连接的配置。
#[derive(Debug, Clone, Deserialize)]
pub struct MysqlConfig {
    pub host: String,
    #[serde(default = "default_mysql_port")]
    pub port: u16,
    #[serde(alias = "username")]
    pub user: String,
    pub password: String,
    #[serde(default)]
    pub database: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default)]
    pub disable_sql_mode: bool,
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout: u32,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u32,
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: default_mysql_port(),
            user: String::new(),
            password: String::new(),
            database: String::new(),
            max_connections: default_max_connections(),
            ssl_mode: default_ssl_mode(),
            disable_sql_mode: false,
            acquire_timeout: default_acquire_timeout(),
            idle_timeout: default_idle_timeout(),
        }
    }
}

fn default_mysql_port() -> u16 {
    3306
}
fn default_max_connections() -> u32 {
    10
}
fn default_acquire_timeout() -> u32 {
    5
}
fn default_idle_timeout() -> u32 {
    60
}
fn default_ssl_mode() -> String {
    "preferred".to_string()
}

impl Validate for MysqlConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.host.is_empty() {
            return Err(Error::ConfigValidation("MySQL host 不能为空".into()));
        }
        if self.database.is_empty() {
            return Err(Error::ConfigValidation("MySQL database 不能为空".into()));
        }
        if self.user.is_empty() {
            return Err(Error::ConfigValidation("MySQL user 不能为空".into()));
        }
        if self.port == 0 {
            return Err(Error::ConfigValidation("MySQL port 不能为 0".into()));
        }
        if self.max_connections == 0 {
            return Err(Error::ConfigValidation(
                "MySQL max_connections 不能为 0".into(),
            ));
        }
        if self.acquire_timeout == 0 {
            return Err(Error::ConfigValidation(
                "MySQL acquire_timeout 不能为 0".into(),
            ));
        }
        if self.idle_timeout == 0 {
            return Err(Error::ConfigValidation(
                "MySQL idle_timeout 不能为 0".into(),
            ));
        }
        let valid_modes = [
            "disabled",
            "disable",
            "off",
            "preferred",
            "required",
            "require",
            "verify-ca",
            "verify_ca",
            "verify-identity",
            "verify_identity",
        ];
        if !valid_modes.contains(&self.ssl_mode.as_str()) {
            return Err(Error::ConfigValidation(format!(
                "MySQL ssl_mode 无效: `{}`，可选: disabled, disable, off, preferred, required, require, verify-ca, verify-identity",
                self.ssl_mode
            )));
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────
// MySQL 子构建器
// ──────────────────────────────────────────────

/// MySQL 单连接构建器，提供链式 API。
pub struct MysqlConfigBuilder(pub(crate) MysqlConfig);

impl MysqlConfigBuilder {
    pub fn host(mut self, v: impl Into<String>) -> Self {
        self.0.host = v.into();
        self
    }
    pub fn port(mut self, v: u16) -> Self {
        self.0.port = v;
        self
    }
    pub fn user(mut self, v: impl Into<String>) -> Self {
        self.0.user = v.into();
        self
    }
    pub fn password(mut self, v: impl Into<String>) -> Self {
        self.0.password = v.into();
        self
    }
    pub fn database(mut self, v: impl Into<String>) -> Self {
        self.0.database = v.into();
        self
    }
    pub fn max_connections(mut self, v: u32) -> Self {
        self.0.max_connections = v;
        self
    }
    pub fn ssl_mode(mut self, v: impl Into<String>) -> Self {
        self.0.ssl_mode = v.into();
        self
    }
    pub fn disable_sql_mode(mut self, v: bool) -> Self {
        self.0.disable_sql_mode = v;
        self
    }
    /// 设置连接超时时间（秒）
    pub fn acquire_timeout(mut self, v: u32) -> Self {
        self.0.acquire_timeout = v;
        self
    }
    /// 设置空闲连接回收时间（秒）
    pub fn idle_timeout(mut self, v: u32) -> Self {
        self.0.idle_timeout = v;
        self
    }
}

// ──────────────────────────────────────────────
// 环境变量解析
// ──────────────────────────────────────────────

const MYSQL_ENV_FIELDS: &[&str] = &[
    "HOST",
    "PORT",
    "USER",
    "PASSWORD",
    "DATABASE",
    "MAX_CONNECTIONS",
    "SSL_MODE",
    "DISABLE_SQL_MODE",
    "ACQUIRE_TIMEOUT",
    "IDLE_TIMEOUT",
];

pub(crate) fn collect_env_mysql(
    prefix: &str,
    existing: &HashMap<String, MysqlConfig>,
) -> ConfigResult<HashMap<String, MysqlConfig>> {
    let mut result = HashMap::new();
    let pfx_upper = prefix.to_uppercase();
    let prefix_mysql = format!("{pfx_upper}_MYSQL_");

    for (key, val) in std::env::vars() {
        let upper = key.to_uppercase();
        let rest = match upper.strip_prefix(&prefix_mysql) {
            Some(r) => r,
            None => continue,
        };

        let (name, field) = match split_env_field(rest, MYSQL_ENV_FIELDS) {
            Some(v) => v,
            None => continue,
        };

        tracing::trace!(key = %key, name = %name, field = %field, "读取 MySQL 环境变量");

        let entry = result
            .entry(name.clone())
            .or_insert_with(|| existing.get(&name).cloned().unwrap_or_default());

        match field {
            "HOST" => entry.host = val,
            "PORT" => {
                entry.port = val.parse().map_err(|e| Error::EnvParse {
                    key: key.clone(),
                    message: format!("PORT: {}", e),
                })?
            }
            "USER" => entry.user = val,
            "PASSWORD" => entry.password = val,
            "DATABASE" => entry.database = val,
            "MAX_CONNECTIONS" => {
                entry.max_connections = val.parse().map_err(|e| Error::EnvParse {
                    key: key.clone(),
                    message: format!("MAX_CONNECTIONS: {}", e),
                })?
            }
            "SSL_MODE" => entry.ssl_mode = val,
            "DISABLE_SQL_MODE" => {
                entry.disable_sql_mode = matches!(val.as_str(), "1" | "true" | "TRUE")
            }
            "ACQUIRE_TIMEOUT" => {
                entry.acquire_timeout = val.parse().map_err(|e| Error::EnvParse {
                    key: key.clone(),
                    message: format!("ACQUIRE_TIMEOUT: {}", e),
                })?
            }
            "IDLE_TIMEOUT" => {
                entry.idle_timeout = val.parse().map_err(|e| Error::EnvParse {
                    key: key.clone(),
                    message: format!("IDLE_TIMEOUT: {}", e),
                })?
            }
            _ => {}
        }
    }
    Ok(result)
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::ConfigBuilder;

    #[test]
    fn validation_rejects_empty_host() {
        let result = ConfigBuilder::new()
            .with_mysql("default", |m| {
                m.host("").user("u").password("p").database("db")
            })
            .build();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, crate::Error::ConfigValidation(ref msg) if msg.contains("host 不能为空"))
        );
    }
}
