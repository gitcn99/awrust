use std::collections::HashMap;

use serde::Deserialize;

use super::Validate;

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
        }
    }
}

fn default_mysql_port() -> u16 {
    3306
}
fn default_max_connections() -> u32 {
    5
}
fn default_ssl_mode() -> String {
    "preferred".to_string()
}

impl Validate for MysqlConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.host.is_empty() {
            anyhow::bail!("MySQL host 不能为空");
        }
        if self.database.is_empty() {
            anyhow::bail!("MySQL database 不能为空");
        }
        if self.user.is_empty() {
            anyhow::bail!("MySQL user 不能为空");
        }
        if self.port == 0 {
            anyhow::bail!("MySQL port 不能为 0");
        }
        if self.max_connections == 0 {
            anyhow::bail!("MySQL max_connections 不能为 0");
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
            anyhow::bail!(
                "MySQL ssl_mode 无效: `{}`，可选: disabled, preferred, required, verify-ca, verify-identity",
                self.ssl_mode
            );
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
}

// ──────────────────────────────────────────────
// 环境变量解析
// ──────────────────────────────────────────────

pub(crate) fn collect_env_mysql(
    prefix: &str,
    existing: &HashMap<String, MysqlConfig>,
) -> anyhow::Result<HashMap<String, MysqlConfig>> {
    let mut result = HashMap::new();
    let pfx_upper = prefix.to_uppercase();

    for (key, val) in std::env::vars() {
        let upper = key.to_uppercase();
        // 匹配 <PREFIX>_MYSQL_<NAME>_<FIELD>
        let rest = match upper.strip_prefix(&format!("{pfx_upper}_MYSQL_")) {
            Some(r) => r,
            None => continue,
        };
        let (name, field) = match rest.rsplit_once('_') {
            Some((n, f)) => (n.to_lowercase(), f),
            None => continue,
        };

        let entry = result
            .entry(name.clone())
            .or_insert_with(|| existing.get(&name).cloned().unwrap_or_default());

        match field {
            "HOST" => entry.host = val,
            "PORT" => {
                entry.port = val
                    .parse()
                    .map_err(|e| anyhow::anyhow!("PORT 解析失败: {}", e))?
            }
            "USER" => entry.user = val,
            "PASSWORD" => entry.password = val,
            "DATABASE" => entry.database = val,
            "MAX_CONNECTIONS" => {
                entry.max_connections = val
                    .parse()
                    .map_err(|e| anyhow::anyhow!("MAX_CONNECTIONS 解析失败: {}", e))?
            }
            "SSL_MODE" => entry.ssl_mode = val,
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
        assert!(result.unwrap_err().to_string().contains("host 不能为空"));
    }
}
