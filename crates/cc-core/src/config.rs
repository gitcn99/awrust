//! TOML 配置文件的数据结构与加载逻辑。
//!
//! 配置示例见 `config/config.toml.example`。

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// MySQL 连接名的抽象，用户可为枚举实现此 trait 以获得编译时检查。
pub trait IntoMysqlName {
    fn into_name(self) -> String;
}

/// Redis 连接名的抽象，用户可为枚举实现此 trait 以获得编译时检查。
pub trait IntoRedisName {
    fn into_name(self) -> String;
}

/// 为 String 实现，保持字符串方式可用。
impl IntoMysqlName for String {
    fn into_name(self) -> String {
        self
    }
}

/// 为 &str 实现，保持字符串方式可用。
impl IntoMysqlName for &str {
    fn into_name(self) -> String {
        self.to_string()
    }
}

/// 为 String 实现，保持字符串方式可用。
impl IntoRedisName for String {
    fn into_name(self) -> String {
        self
    }
}

/// 为 &str 实现，保持字符串方式可用。
impl IntoRedisName for &str {
    fn into_name(self) -> String {
        self.to_string()
    }
}

/// 单个 MySQL 连接的配置。
#[derive(Debug, Clone, Deserialize)]
pub struct MysqlConfig {
    /// 主机名
    pub host: String,
    /// 端口
    #[serde(default = "default_mysql_port")]
    pub port: u16,
    /// 用户名（兼容 `username` 写法）
    #[serde(alias = "username")]
    pub user: String,
    /// 密码
    pub password: String,
    /// 默认库（schema）；留空表示不指定
    #[serde(default)]
    pub database: String,
    /// 连接池最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// TLS 模式：disabled | preferred | required | verify-ca | verify-identity
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
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

/// 单个 Redis 连接的配置。
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// 形如 `redis://[:password@]host:port[/db]` 的连接串
    pub url: String,
}

/// 整个配置文件：多个命名 MySQL / Redis 连接。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// 名字 -> MySQL 配置
    #[serde(default)]
    pub mysql: HashMap<String, MysqlConfig>,
    /// 名字 -> Redis 配置
    #[serde(default)]
    pub redis: HashMap<String, RedisConfig>,
}

impl Config {
    /// 从 TOML 文件加载配置。
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("读取配置文件 {} 失败: {}", path.display(), e))?;
        let cfg: Config = toml::from_str(&text)
            .map_err(|e| anyhow::anyhow!("解析配置文件 {} 失败: {}", path.display(), e))?;
        Ok(cfg)
    }

    /// 按名取 MySQL 配置。
    pub fn mysql(&self, name: &str) -> Option<&MysqlConfig> {
        self.mysql.get(name)
    }

    /// 按名取 Redis 配置。
    pub fn redis(&self, name: &str) -> Option<&RedisConfig> {
        self.redis.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config() {
        let toml = r#"
            [mysql.default]
            host = "h1"
            port = 3306
            user = "u1"
            password = "p1"
            database = "db1"
            max_connections = 1
            ssl_mode = "preferred"

            [redis.default]
            url = "redis://127.0.0.1:6379"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();

        let d = cfg.mysql("default").unwrap();
        assert_eq!(d.host, "h1");
        assert_eq!(d.port, 3306);

        assert_eq!(cfg.redis("default").unwrap().url, "redis://127.0.0.1:6379");
    }
}
