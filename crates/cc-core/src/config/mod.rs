//! 分层配置系统：支持文件 → 环境变量 → 程序化覆盖的合并策略。
//!
//! # 配置来源优先级（从低到高）
//!
//! 1. **配置文件** — `Config::from_file("config.toml")` 等
//! 2. **环境变量** — `CC_MYSQL_<name>_<field>=value` 格式
//! 3. **程序化覆盖** — `ConfigBuilder::with_mysql()` / `with_redis()`
//!
//! # 环境变量格式
//!
//! ```text
//! CC_MYSQL_<NAME>_HOST=127.0.0.1
//! CC_MYSQL_<NAME>_PORT=3306
//! CC_MYSQL_<NAME>_USER=root
//! CC_MYSQL_<NAME>_PASSWORD=secret
//! CC_MYSQL_<NAME>_DATABASE=mydb
//!
//! CC_REDIS_<NAME>_URL=redis://127.0.0.1:6379
//! ```

mod mysql;
mod redis;

pub use mysql::{MysqlConfig, MysqlConfigBuilder};
pub use redis::{RedisConfig, RedisConfigBuilder};

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{ConfigResult, Error};

/// 默认环境变量前缀。
pub const DEFAULT_ENV_PREFIX: &str = "CC";

/// 从 `NAME_FIELD` 格式的字符串中，按已知字段名列表从右匹配，拆分出 (name, field)。
pub(crate) fn split_env_field<'a>(
    rest: &'a str,
    known_fields: &[&'a str],
) -> Option<(String, &'a str)> {
    for &field in known_fields {
        let suffix = format!("_{field}");
        if let Some(name) = rest.strip_suffix(&suffix) {
            if !name.is_empty() {
                return Some((name.to_lowercase(), field));
            }
        }
    }
    None
}

// ──────────────────────────────────────────────
// 连接名抽象
// ──────────────────────────────────────────────

/// MySQL 连接名的抽象，用户可为枚举实现此 trait 以获得编译时检查。
pub trait IntoMysqlName {
    fn into_name(self) -> String;
}

/// Redis 连接名的抽象，用户可为枚举实现此 trait 以获得编译时检查。
pub trait IntoRedisName {
    fn into_name(self) -> String;
}

impl IntoMysqlName for String {
    fn into_name(self) -> String {
        self
    }
}

impl IntoMysqlName for &String {
    fn into_name(self) -> String {
        self.clone()
    }
}

impl IntoMysqlName for &str {
    fn into_name(self) -> String {
        self.to_string()
    }
}

impl IntoRedisName for String {
    fn into_name(self) -> String {
        self
    }
}

impl IntoRedisName for &String {
    fn into_name(self) -> String {
        self.clone()
    }
}

impl IntoRedisName for &str {
    fn into_name(self) -> String {
        self.to_string()
    }
}

// ──────────────────────────────────────────────
// 验证 trait
// ──────────────────────────────────────────────

/// 配置项验证。`Config::build()` 会自动调用。
pub trait Validate {
    fn validate(&self) -> ConfigResult<()>;
}

// ──────────────────────────────────────────────
// Config — 顶层容器
// ──────────────────────────────────────────────

/// 整个配置：多个命名 MySQL / Redis 连接。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub mysql: HashMap<String, MysqlConfig>,
    #[serde(default)]
    pub redis: HashMap<String, RedisConfig>,
}

impl Config {
    /// 按名取 MySQL 配置。
    pub fn mysql(&self, name: &str) -> Option<&MysqlConfig> {
        self.mysql.get(name)
    }

    /// 按名取 Redis 配置。
    pub fn redis(&self, name: &str) -> Option<&RedisConfig> {
        self.redis.get(name)
    }

    /// 获取所有 MySQL 连接名。
    pub fn mysql_names(&self) -> impl Iterator<Item = &str> {
        self.mysql.keys().map(String::as_str)
    }

    /// 获取所有 Redis 连接名。
    pub fn redis_names(&self) -> impl Iterator<Item = &str> {
        self.redis.keys().map(String::as_str)
    }
}

impl Validate for Config {
    fn validate(&self) -> ConfigResult<()> {
        for (name, mc) in &self.mysql {
            mc.validate()
                .map_err(|e| Error::ConfigValidation(format!("MySQL[{}]: {}", name, e)))?;
        }
        for (name, rc) in &self.redis {
            rc.validate()
                .map_err(|e| Error::ConfigValidation(format!("Redis[{}]: {}", name, e)))?;
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────
// ConfigBuilder
// ──────────────────────────────────────────────

/// 分层配置构建器，支持文件 → 环境变量 → 程序化覆盖。
///
/// ```rust
/// use cc_core::ConfigBuilder;
///
/// let cfg = ConfigBuilder::new()
///     .with_mysql("default", |m| m.host("127.0.0.1").user("root").password("pw").database("mydb"))
///     .with_redis("cache", |r| r.url("redis://127.0.0.1:6379"))
///     .build()
///     .unwrap();
/// ```
pub struct ConfigBuilder {
    mysql: HashMap<String, MysqlConfig>,
    redis: HashMap<String, RedisConfig>,
    env_prefix: String,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        tracing::debug!("创建 ConfigBuilder");
        Self {
            mysql: HashMap::new(),
            redis: HashMap::new(),
            env_prefix: DEFAULT_ENV_PREFIX.to_string(),
        }
    }

    /// 从配置文件创建 ConfigBuilder，根据扩展名自动选择解析器。
    pub fn from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        Self::new().with_file(path)
    }

    /// 从 TOML 字符串创建 ConfigBuilder。
    #[cfg(feature = "config-toml")]
    pub fn from_toml(toml_str: &str) -> ConfigResult<Self> {
        Self::new().with_toml(toml_str)
    }

    /// 从 YAML 字符串创建 ConfigBuilder（需要 `config-yaml` feature）。
    #[cfg(feature = "config-yaml")]
    pub fn from_yaml(yaml_str: &str) -> ConfigResult<Self> {
        Self::new().with_yaml(yaml_str)
    }

    /// 从 JSON 字符串创建 ConfigBuilder（需要 `config-json` feature）。
    #[cfg(feature = "config-json")]
    pub fn from_json(json_str: &str) -> ConfigResult<Self> {
        Self::new().with_json(json_str)
    }

    /// 从环境变量创建 ConfigBuilder。
    pub fn from_env() -> ConfigResult<Self> {
        Self::new().with_env()
    }

    /// 设置环境变量前缀（默认 "CC"）。
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// 从配置文件加载，根据扩展名自动选择解析器。
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> ConfigResult<Self> {
        let path = path.as_ref();
        tracing::info!(path = %path.display(), "加载配置文件");
        let text = std::fs::read_to_string(path).map_err(|e| Error::ConfigFileRead {
            path: path.display().to_string(),
            source: e,
        })?;

        match path.extension().and_then(|e| e.to_str()) {
            #[cfg(feature = "config-toml")]
            Some("toml") => self.with_toml(&text),
            #[cfg(feature = "config-yaml")]
            Some("yaml") | Some("yml") => self.with_yaml(&text),
            #[cfg(feature = "config-json")]
            Some("json") => self.with_json(&text),
            Some(ext) => Err(Error::ConfigParse {
                format: ext.to_string(),
                message: "不支持的配置文件格式".into(),
            }),
            None => Err(Error::ConfigParse {
                format: "unknown".into(),
                message: "无法识别配置文件扩展名".into(),
            }),
        }
    }

    /// 从内联 TOML 字符串加载（方便测试和动态配置）。
    #[cfg(feature = "config-toml")]
    pub fn with_toml(self, toml_str: &str) -> ConfigResult<Self> {
        tracing::debug!("解析 TOML 配置");
        let file_cfg: Config = toml::from_str(toml_str)?;
        Ok(self.merge(file_cfg))
    }

    /// 从内联 YAML 字符串加载（需要 `config-yaml` feature）。
    #[cfg(feature = "config-yaml")]
    pub fn with_yaml(self, yaml_str: &str) -> ConfigResult<Self> {
        tracing::debug!("解析 YAML 配置");
        let file_cfg: Config = yaml_serde::from_str(yaml_str)?;
        Ok(self.merge(file_cfg))
    }

    /// 从内联 JSON 字符串加载（需要 `config-json` feature）。
    #[cfg(feature = "config-json")]
    pub fn with_json(self, json_str: &str) -> ConfigResult<Self> {
        tracing::debug!("解析 JSON 配置");
        let file_cfg: Config = serde_json::from_str(json_str)?;
        Ok(self.merge(file_cfg))
    }

    /// 读取环境变量覆盖。格式：`<PREFIX>_MYSQL_<NAME>_<FIELD>` / `<PREFIX>_REDIS_<NAME>_<FIELD>`
    pub fn with_env(mut self) -> ConfigResult<Self> {
        let prefix = &self.env_prefix;
        tracing::debug!(prefix = %prefix, "读取环境变量配置");
        self.mysql
            .extend(mysql::collect_env_mysql(prefix, &self.mysql)?);
        self.redis
            .extend(redis::collect_env_redis(prefix, &self.redis)?);
        Ok(self)
    }

    /// 程序化添加 / 覆盖单个 MySQL 连接。
    pub fn with_mysql(
        mut self,
        name: impl Into<String>,
        f: impl FnOnce(MysqlConfigBuilder) -> MysqlConfigBuilder,
    ) -> Self {
        let name = name.into();
        tracing::debug!(name = %name, "配置 MySQL 连接");
        let base = self.mysql.remove(&name).unwrap_or_default();
        let cfg = f(MysqlConfigBuilder(base)).0;
        self.mysql.insert(name, cfg);
        self
    }

    /// 程序化添加 / 覆盖单个 Redis 连接。
    pub fn with_redis(
        mut self,
        name: impl Into<String>,
        f: impl FnOnce(RedisConfigBuilder) -> RedisConfigBuilder,
    ) -> Self {
        let name = name.into();
        tracing::debug!(name = %name, "配置 Redis 连接");
        let base = self.redis.remove(&name).unwrap_or_default();
        let cfg = f(RedisConfigBuilder(base)).0;
        self.redis.insert(name, cfg);
        self
    }

    /// 合并另一个 Config（other 覆盖 self）。
    pub fn merge(mut self, other: Config) -> Self {
        tracing::debug!(
            mysql_count = other.mysql.len(),
            redis_count = other.redis.len(),
            "合并配置"
        );
        self.mysql.extend(other.mysql);
        self.redis.extend(other.redis);
        self
    }

    /// 构建最终配置并验证。
    pub fn build(self) -> ConfigResult<Config> {
        let cfg = Config {
            mysql: self.mysql,
            redis: self.redis,
        };
        tracing::info!(
            mysql_count = cfg.mysql.len(),
            redis_count = cfg.redis.len(),
            "配置构建完成"
        );
        cfg.validate()?;
        Ok(cfg)
    }
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "config-toml")]
    #[test]
    fn parses_toml() {
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
        let cfg = ConfigBuilder::from_toml(toml).unwrap().build().unwrap();
        let d = cfg.mysql("default").unwrap();
        assert_eq!(d.host, "h1");
        assert_eq!(d.port, 3306);
        assert_eq!(cfg.redis("default").unwrap().url, "redis://127.0.0.1:6379");
    }

    #[test]
    fn builder_basic() {
        let cfg = ConfigBuilder::new()
            .with_mysql("default", |m| {
                m.host("10.0.0.1")
                    .port(3307)
                    .user("root")
                    .password("pw")
                    .database("test")
            })
            .with_redis("cache", |r| r.url("redis://127.0.0.1:6380"))
            .build()
            .unwrap();

        assert_eq!(cfg.mysql("default").unwrap().port, 3307);
        assert_eq!(cfg.redis("cache").unwrap().url, "redis://127.0.0.1:6380");
    }

    #[cfg(feature = "config-toml")]
    #[test]
    fn builder_merge_file() {
        let toml = r#"
            [mysql.default]
            host = "file-host"
            user = "file-user"
            password = "file-pw"
            database = "file-db"
        "#;
        let cfg = ConfigBuilder::new()
            .with_toml(toml)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(cfg.mysql("default").unwrap().host, "file-host");
    }

    #[test]
    fn from_map_works() {
        let mut mysql = HashMap::new();
        mysql.insert(
            "default".into(),
            MysqlConfig {
                host: "h".into(),
                port: 3306,
                user: "u".into(),
                password: "p".into(),
                database: "db".into(),
                max_connections: 5,
                ssl_mode: "preferred".into(),
                disable_sql_mode: false,
                acquire_timeout: 5,
                idle_timeout: 60,
            },
        );
        let cfg = ConfigBuilder::new()
            .merge(Config {
                mysql,
                redis: HashMap::new(),
            })
            .build()
            .unwrap();
        assert_eq!(cfg.mysql("default").unwrap().host, "h");
    }

    #[test]
    fn env_prefix_override() {
        std::env::set_var("TEST_CC_MYSQL_DEFAULT_HOST", "env-host");
        std::env::set_var("TEST_CC_REDIS_DEFAULT_URL", "redis://env:6379");

        let cfg = ConfigBuilder::new()
            .env_prefix("TEST_CC")
            .with_mysql("default", |m| m.user("u").password("p").database("db"))
            .with_env()
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(cfg.mysql("default").unwrap().host, "env-host");
        assert_eq!(cfg.redis("default").unwrap().url, "redis://env:6379");

        std::env::remove_var("TEST_CC_MYSQL_DEFAULT_HOST");
        std::env::remove_var("TEST_CC_REDIS_DEFAULT_URL");
    }

    #[cfg(feature = "config-toml")]
    #[test]
    fn from_file_works() {
        let toml = r#"
            [mysql.default]
            host = "file-host"
            user = "file-user"
            password = "file-pw"
            database = "file-db"
        "#;
        let cfg = ConfigBuilder::from_toml(toml).unwrap().build().unwrap();
        assert_eq!(cfg.mysql("default").unwrap().host, "file-host");
    }

    #[test]
    fn from_env_works() {
        std::env::set_var("ENV_WORKS_CC_MYSQL_DEFAULT_HOST", "env-host");

        let cfg = ConfigBuilder::new()
            .env_prefix("ENV_WORKS_CC")
            .with_mysql("default", |m| m.user("u").password("p").database("db"))
            .with_env()
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(cfg.mysql("default").unwrap().host, "env-host");

        std::env::remove_var("ENV_WORKS_CC_MYSQL_DEFAULT_HOST");
    }

    #[test]
    fn mysql_names_iterator() {
        let cfg = ConfigBuilder::new()
            .with_mysql("primary", |m| {
                m.host("h1").user("u").password("p").database("d")
            })
            .with_mysql("replica", |m| {
                m.host("h2").user("u").password("p").database("d")
            })
            .build()
            .unwrap();

        let mut names: Vec<_> = cfg.mysql_names().collect();
        names.sort();
        assert_eq!(names, vec!["primary", "replica"]);
    }
}
