//! 分层配置系统：支持文件 → 环境变量 → 程序化覆盖的合并策略。
//!
//! # 配置来源优先级（从低到高）
//!
//! 1. **ConfigBuilder 默认值** — Builder 自带的默认值
//! 2. **TOML 文件** — `Config::from_file("config.toml")`
//! 3. **环境变量** — `CC_MYSQL_<name>_<field>=value` 格式
//! 4. **程序化覆盖** — `ConfigBuilder::with_mysql()` / `with_redis()`
//!
//! # 环境变量格式
//!
//! ```text
//! CC_MYSQL_<NAME>_HOST=127.0.0.1
//! CC_MYSQL_<NAME>_PORT=3306
//! CC_MYSQL_<NAME>_USER=root
//! CC_MYSQL_<NAME>_PASSWORD=secret
//! CC_MYSQL_<NAME>_DATABASE=mydb
//! CC_MYSQL_<NAME>_MAX_CONNECTIONS=10
//! CC_MYSQL_<NAME>_SSL_MODE=preferred
//!
//! CC_REDIS_<NAME>_URL=redis://localhost:6379
//! ```

mod mysql;
mod redis;

pub use mysql::{MysqlConfig, MysqlConfigBuilder};
pub use redis::{RedisConfig, RedisConfigBuilder};

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

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
    fn validate(&self) -> anyhow::Result<()>;
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
}

impl Validate for Config {
    fn validate(&self) -> anyhow::Result<()> {
        for (name, mc) in &self.mysql {
            mc.validate()
                .map_err(|e| anyhow::anyhow!("MySQL[{}]: {}", name, e))?;
        }
        for (name, rc) in &self.redis {
            rc.validate()
                .map_err(|e| anyhow::anyhow!("Redis[{}]: {}", name, e))?;
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
///     .with_redis("cache", |r| r.url("redis://localhost:6379"))
///     .build()
///     .unwrap();
/// ```
pub struct ConfigBuilder {
    mysql: HashMap<String, MysqlConfig>,
    redis: HashMap<String, RedisConfig>,
    env_prefix: String,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            mysql: HashMap::new(),
            redis: HashMap::new(),
            env_prefix: "CC".to_string(),
        }
    }

    /// 从 TOML 文件创建 ConfigBuilder。
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Self::new().with_file(path)
    }

    /// 从 TOML 字符串创建 ConfigBuilder。
    pub fn from_toml(toml_str: &str) -> anyhow::Result<Self> {
        Self::new().with_toml(toml_str)
    }

    /// 从环境变量创建 ConfigBuilder。
    pub fn from_env() -> anyhow::Result<Self> {
        Self::new().with_env()
    }

    /// 设置环境变量前缀（默认 "CC"）。
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// 从 TOML 文件加载（与已有配置合并，文件值覆盖已有值）。
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("读取配置文件 {} 失败: {}", path.display(), e))?;
        self.with_toml(&text)
    }

    /// 从内联 TOML 字符串加载（方便测试和动态配置）。
    pub fn with_toml(self, toml_str: &str) -> anyhow::Result<Self> {
        let file_cfg: Config =
            toml::from_str(toml_str).map_err(|e| anyhow::anyhow!("解析 TOML 失败: {}", e))?;
        Ok(self.merge(file_cfg))
    }

    /// 读取环境变量覆盖。格式：`<PREFIX>_MYSQL_<NAME>_<FIELD>` / `<PREFIX>_REDIS_<NAME>_<FIELD>`
    pub fn with_env(mut self) -> anyhow::Result<Self> {
        let prefix = &self.env_prefix;
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
        let base = self.redis.remove(&name).unwrap_or_default();
        let cfg = f(RedisConfigBuilder(base)).0;
        self.redis.insert(name, cfg);
        self
    }

    /// 合并另一个 Config（other 覆盖 self）。
    pub fn merge(mut self, other: Config) -> Self {
        self.mysql.extend(other.mysql);
        self.redis.extend(other.redis);
        self
    }

    /// 构建最终配置并验证。
    pub fn build(self) -> anyhow::Result<Config> {
        let cfg = Config {
            mysql: self.mysql,
            redis: self.redis,
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
            .with_redis("cache", |r| r.url("redis://localhost:6380"))
            .build()
            .unwrap();

        assert_eq!(cfg.mysql("default").unwrap().port, 3307);
        assert_eq!(cfg.redis("cache").unwrap().url, "redis://localhost:6380");
    }

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
        std::env::set_var("TEST_CC_MYSQL_DEFAULT_HOST", "env-host");

        let cfg = ConfigBuilder::new()
            .env_prefix("TEST_CC")
            .with_mysql("default", |m| m.user("u").password("p").database("db"))
            .with_env()
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(cfg.mysql("default").unwrap().host, "env-host");

        std::env::remove_var("TEST_CC_MYSQL_DEFAULT_HOST");
    }
}
