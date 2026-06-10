//! 命名模式配置系统：每个环境一个独立配置文件 → 环境变量 → 程序化覆盖。
//!
//! # 配置来源优先级（从低到高）
//!
//! 1. **编译模式默认值** — `debug_assertions` 开启时默认 `dev`，关闭时默认 `online`
//! 2. **命名模式配置文件** — `config/config.dev.toml` / `config/config.online.toml` / `config/config.<任意名>.<ext>`
//! 3. **环境变量** — `CC_MODE=dev`、`CC_MYSQL_<name>_<field>=value`
//! 4. **程序化覆盖** — `ConfigBuilder::with_mysql()` / `with_redis()`
//!
//! # 配置布局
//!
//! 每个环境使用独立的完整配置文件，支持 TOML / YAML / JSON 格式（按 feature 启用）：
//!
//! ```text
//! config/config.dev.toml      ← 开发环境（完整独立）
//! config/config.online.toml   ← 线上环境（完整独立）
//! config/config.staging.toml  ← 自定义命名（完整独立）
//! ```
//!
//! `ConfigBuilder::new()` 会自动确定运行模式并加载 `config/config.<mode>.<ext>`（按 feature 尝试 toml/yaml/json）。
//!
//! # 环境变量格式
//!
//! ```text
//! CC_MODE=dev
//!
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
mod tracing;

pub use mysql::{MysqlConfig, MysqlConfigBuilder};
pub use redis::{RedisConfig, RedisConfigBuilder};
pub use tracing::{TracingConfig, TracingConfigBuilder};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// 在配置目录下，按已启用的格式查找 `config.<mode>.<ext>` 文件。
///
/// 返回第一个存在的文件路径；若无匹配则返回所有候选路径（用于错误信息）。
fn find_mode_config(dir: &Path, mode: &str) -> Result<PathBuf, Vec<PathBuf>> {
    let mut tried = Vec::new();
    #[cfg(feature = "config-toml")]
    {
        let p = dir.join(format!("config.{mode}.toml"));
        if p.exists() {
            return Ok(p);
        }
        tried.push(p);
    }
    #[cfg(feature = "config-yaml")]
    {
        let p = dir.join(format!("config.{mode}.yaml"));
        if p.exists() {
            return Ok(p);
        }
        tried.push(p);
        let p = dir.join(format!("config.{mode}.yml"));
        if p.exists() {
            return Ok(p);
        }
        tried.push(p);
    }
    #[cfg(feature = "config-json")]
    {
        let p = dir.join(format!("config.{mode}.json"));
        if p.exists() {
            return Ok(p);
        }
        tried.push(p);
    }
    Err(tried)
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

/// 整个配置：多个命名 MySQL / Redis 连接 + Tracing 日志配置。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// 当前运行模式（如 "dev"、"online"），由 ConfigBuilder 设置，不参与序列化。
    #[serde(skip)]
    pub mode: Option<String>,
    #[serde(default)]
    pub mysql: HashMap<String, MysqlConfig>,
    #[serde(default)]
    pub redis: HashMap<String, RedisConfig>,
    #[serde(default)]
    pub tracing: TracingConfig,
}

impl Config {
    /// 获取当前运行模式。
    pub fn mode(&self) -> Option<&str> {
        self.mode.as_deref()
    }

    /// 按名取 MySQL 配置。
    pub fn mysql(&self, name: &str) -> Option<&MysqlConfig> {
        self.mysql.get(name)
    }

    /// 按名取 Redis 配置。
    pub fn redis(&self, name: &str) -> Option<&RedisConfig> {
        self.redis.get(name)
    }

    /// 获取 Tracing 配置。
    pub fn tracing(&self) -> &TracingConfig {
        &self.tracing
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
        self.tracing
            .validate()
            .map_err(|e| Error::ConfigValidation(format!("Tracing: {}", e)))?;
        Ok(())
    }
}

// ──────────────────────────────────────────────
// ConfigBuilder
// ──────────────────────────────────────────────

/// 命名模式配置构建器：自动加载 `config/config.<mode>.<ext>` → 环境变量 → 程序化覆盖。
///
/// 使用自动加载（需存在配置文件）：
/// ```rust,no_run
/// use cc_core::{ConfigBuilder, ConfigResult};
///
/// fn main() -> ConfigResult<()> {
///     let cfg = ConfigBuilder::new()?
///         .with_mysql("default", |m| m.host("127.0.0.1").user("root").password("pw").database("mydb"))
///         .with_redis("cache", |r| r.url("redis://127.0.0.1:6379"))
///         .build()?;
///     Ok(())
/// }
/// ```
///
/// 仅程序化构建（无需配置文件）：
/// ```rust
/// use cc_core::{ConfigBuilder, ConfigResult};
///
/// fn main() -> ConfigResult<()> {
///     let cfg = ConfigBuilder::empty()
///         .with_mysql("default", |m| m.host("127.0.0.1").user("root").password("pw").database("mydb"))
///         .with_redis("cache", |r| r.url("redis://127.0.0.1:6379"))
///         .build()?;
///     Ok(())
/// }
/// ```
pub struct ConfigBuilder {
    mode: Option<String>,
    mysql: HashMap<String, MysqlConfig>,
    redis: HashMap<String, RedisConfig>,
    tracing: TracingConfig,
    env_prefix: String,
}

impl ConfigBuilder {
    /// 创建 ConfigBuilder，自动确定运行模式并加载 `config/config.<mode>.toml`。
    pub fn new() -> ConfigResult<Self> {
        ::tracing::debug!("创建 ConfigBuilder");
        let mut builder = Self::empty();
        #[cfg(debug_assertions)]
        {
            builder = builder.with_mode("dev");
        }
        #[cfg(not(debug_assertions))]
        {
            builder = builder.with_mode("online");
        }
        let prefix = builder.env_prefix.clone();
        if let Ok(mode) = std::env::var(format!("{}_MODE", prefix)) {
            if !mode.is_empty() {
                builder = builder.with_mode(mode);
            }
        }
        let mode = builder.mode.clone().unwrap_or_else(|| "dev".into());
        let config_dir = Path::new("config");
        let path = match find_mode_config(config_dir, &mode) {
            Ok(p) => p,
            Err(tried) => {
                let paths = tried
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Error::ModeConfigNotFound {
                    path: paths,
                    mode,
                    prefix,
                });
            }
        };
        builder.with_file(&path)
    }

    pub fn empty() -> Self {
        Self {
            mode: None,
            mysql: HashMap::new(),
            redis: HashMap::new(),
            tracing: TracingConfig::default(),
            env_prefix: DEFAULT_ENV_PREFIX.to_string(),
        }
    }

    /// 从 TOML 字符串创建 ConfigBuilder。
    #[cfg(feature = "config-toml")]
    pub fn from_toml(toml_str: &str) -> ConfigResult<Self> {
        Self::empty().with_toml(toml_str)
    }

    /// 从 YAML 字符串创建 ConfigBuilder（需要 `config-yaml` feature）。
    #[cfg(feature = "config-yaml")]
    pub fn from_yaml(yaml_str: &str) -> ConfigResult<Self> {
        Self::empty().with_yaml(yaml_str)
    }

    /// 从 JSON 字符串创建 ConfigBuilder（需要 `config-json` feature）。
    #[cfg(feature = "config-json")]
    pub fn from_json(json_str: &str) -> ConfigResult<Self> {
        Self::empty().with_json(json_str)
    }

    /// 从环境变量创建 ConfigBuilder。
    pub fn from_env() -> ConfigResult<Self> {
        Self::empty().with_env()
    }

    /// 设置环境变量前缀（默认 "CC"）。
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// 设置运行模式（如 `"dev"`、`"online"`）。
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        let mode = mode.into();
        ::tracing::info!(mode = %mode, "设置运行模式");
        self.mode = Some(mode);
        self
    }

    /// 获取当前运行模式。
    pub fn mode(&self) -> Option<&str> {
        self.mode.as_deref()
    }

    /// 从配置文件加载，根据扩展名自动选择解析器。
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> ConfigResult<Self> {
        let path = path.as_ref();
        ::tracing::info!(path = %path.display(), "加载配置文件");
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
        ::tracing::debug!("解析 TOML 配置");
        let file_cfg: Config = toml::from_str(toml_str)?;
        Ok(self.merge(file_cfg))
    }

    /// 从内联 YAML 字符串加载（需要 `config-yaml` feature）。
    #[cfg(feature = "config-yaml")]
    pub fn with_yaml(self, yaml_str: &str) -> ConfigResult<Self> {
        ::tracing::debug!("解析 YAML 配置");
        let file_cfg: Config = yaml_serde::from_str(yaml_str)?;
        Ok(self.merge(file_cfg))
    }

    /// 从内联 JSON 字符串加载（需要 `config-json` feature）。
    #[cfg(feature = "config-json")]
    pub fn with_json(self, json_str: &str) -> ConfigResult<Self> {
        ::tracing::debug!("解析 JSON 配置");
        let file_cfg: Config = serde_json::from_str(json_str)?;
        Ok(self.merge(file_cfg))
    }

    /// 读取环境变量覆盖。格式：`<PREFIX>_MYSQL_<NAME>_<FIELD>` / `<PREFIX>_REDIS_<NAME>_<FIELD>` / `<PREFIX>_TRACING_<FIELD>`
    pub fn with_env(mut self) -> ConfigResult<Self> {
        let prefix = &self.env_prefix;
        ::tracing::debug!(prefix = %prefix, "读取环境变量配置");
        self.mysql
            .extend(mysql::collect_env_mysql(prefix, &self.mysql)?);
        self.redis
            .extend(redis::collect_env_redis(prefix, &self.redis)?);
        self.tracing = tracing::collect_env_tracing(prefix, &self.tracing)?;
        Ok(self)
    }

    /// 程序化添加 / 覆盖单个 MySQL 连接。
    pub fn with_mysql(
        mut self,
        name: impl Into<String>,
        f: impl FnOnce(MysqlConfigBuilder) -> MysqlConfigBuilder,
    ) -> Self {
        let name = name.into();
        ::tracing::debug!(name = %name, "配置 MySQL 连接");
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
        ::tracing::debug!(name = %name, "配置 Redis 连接");
        let base = self.redis.remove(&name).unwrap_or_default();
        let cfg = f(RedisConfigBuilder(base)).0;
        self.redis.insert(name, cfg);
        self
    }

    /// 程序化添加 / 覆盖 Tracing 配置。
    pub fn with_tracing(
        mut self,
        f: impl FnOnce(TracingConfigBuilder) -> TracingConfigBuilder,
    ) -> Self {
        ::tracing::debug!("配置 Tracing");
        let base = std::mem::take(&mut self.tracing);
        self.tracing = f(TracingConfigBuilder(base)).0;
        self
    }

    /// 合并另一个 Config（other 覆盖 self）。
    pub fn merge(mut self, other: Config) -> Self {
        ::tracing::debug!(
            mysql_count = other.mysql.len(),
            redis_count = other.redis.len(),
            "合并配置"
        );
        self.mysql.extend(other.mysql);
        self.redis.extend(other.redis);
        self.tracing = other.tracing;
        self
    }

    /// 构建最终配置并验证。
    pub fn build(self) -> ConfigResult<Config> {
        let cfg = Config {
            mode: self.mode,
            mysql: self.mysql,
            redis: self.redis,
            tracing: self.tracing,
        };
        ::tracing::info!(
            mode = ?cfg.mode,
            mysql_count = cfg.mysql.len(),
            redis_count = cfg.redis.len(),
            tracing_level = %cfg.tracing.level,
            tracing_format = %cfg.tracing.format,
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

    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct SetCurrentDir {
        prev: std::path::PathBuf,
    }

    impl SetCurrentDir {
        fn new(dir: &std::path::Path) -> Self {
            let prev = std::env::current_dir().unwrap();
            std::env::set_current_dir(dir).unwrap();
            Self { prev }
        }
    }

    impl Drop for SetCurrentDir {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.prev);
        }
    }

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
        let cfg = ConfigBuilder::empty()
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
        let cfg = ConfigBuilder::empty()
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
        let cfg = ConfigBuilder::empty()
            .merge(Config {
                mode: None,
                mysql,
                redis: HashMap::new(),
                tracing: TracingConfig::default(),
            })
            .build()
            .unwrap();
        assert_eq!(cfg.mysql("default").unwrap().host, "h");
    }

    #[test]
    fn env_prefix_override() {
        std::env::set_var("TEST_CC_MYSQL_DEFAULT_HOST", "env-host");
        std::env::set_var("TEST_CC_REDIS_DEFAULT_URL", "redis://env:6379");

        let cfg = ConfigBuilder::empty()
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
    fn from_toml_works() {
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

        let cfg = ConfigBuilder::empty()
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
        let cfg = ConfigBuilder::empty()
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

    #[cfg(feature = "config-toml")]
    #[test]
    fn new_loads_mode_file() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let dir = std::env::temp_dir().join("cc_test_new_loads");
        let _ = std::fs::remove_dir_all(&dir);
        let config_dir = dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("config.dev.toml"),
            r#"
                [mysql.default]
                host = "dev-host"
                port = 3307
                user = "u"
                password = "p"
                database = "dev_db"
            "#,
        )
        .unwrap();

        std::env::remove_var("CC_MODE");
        let _guard = SetCurrentDir::new(&dir);
        let cfg = ConfigBuilder::new().unwrap().build().unwrap();

        assert_eq!(cfg.mode(), Some("dev"));
        assert_eq!(cfg.mysql("default").unwrap().host, "dev-host");
        assert_eq!(cfg.mysql("default").unwrap().port, 3307);

        drop(_guard);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(feature = "config-toml")]
    #[test]
    fn new_missing_mode_errors() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let dir = std::env::temp_dir().join("cc_test_new_missing");
        let _ = std::fs::remove_dir_all(&dir);
        let config_dir = dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        let _guard = SetCurrentDir::new(&dir);
        std::env::set_var("CC_MODE", "dev");
        let result = ConfigBuilder::new();
        std::env::remove_var("CC_MODE");
        match result {
            Err(Error::ModeConfigNotFound { mode, path, .. }) => {
                assert_eq!(mode, "dev");
                assert!(
                    path.contains("config.dev.toml"),
                    "unexpected path: {}",
                    path
                );
            }
            other => panic!("expected Err(ModeConfigNotFound), got {:?}", other.is_ok()),
        }

        drop(_guard);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(feature = "config-toml")]
    #[test]
    fn new_env_mode_override() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let dir = std::env::temp_dir().join("cc_test_new_env");
        let _ = std::fs::remove_dir_all(&dir);
        let config_dir = dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("config.online.toml"),
            r#"
                [mysql.default]
                host = "online-host"
                port = 3306
                user = "u"
                password = "p"
                database = "db"

                [tracing]
                level = "warn"
                format = "json"
            "#,
        )
        .unwrap();

        let _guard = SetCurrentDir::new(&dir);
        std::env::set_var("CC_MODE", "online");
        let cfg = ConfigBuilder::new().unwrap().build().unwrap();
        std::env::remove_var("CC_MODE");

        assert_eq!(cfg.mode(), Some("online"));
        assert_eq!(cfg.mysql("default").unwrap().host, "online-host");
        assert_eq!(cfg.tracing.level, "warn");
        assert_eq!(cfg.tracing.format, "json");

        drop(_guard);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
