//! 结构化错误类型，统一库边界错误处理。

/// cc-core 统一错误类型。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ── 配置相关 ──
    /// 配置文件读取失败。
    #[error("读取配置文件 {path} 失败: {source}")]
    ConfigFileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// 配置解析失败（TOML / YAML / JSON）。
    #[error("解析 {format} 配置失败: {message}")]
    ConfigParse { format: String, message: String },

    /// 配置验证失败。
    #[error("配置验证失败: {0}")]
    ConfigValidation(String),

    // ── MySQL 相关 ──
    /// MySQL 连接建立失败。
    #[cfg(feature = "mysql")]
    #[error("连接 MySQL({host}:{port}) 失败: {source}")]
    MysqlConnect {
        host: String,
        port: u16,
        #[source]
        source: sqlx::Error,
    },

    /// MySQL 连接池操作失败。
    #[cfg(feature = "mysql")]
    #[error("MySQL 连接池操作失败: {0}")]
    MysqlPool(#[from] sqlx::Error),

    /// MySQL 连接名未找到。
    #[cfg(feature = "mysql")]
    #[error("未找到名为 `{name}` 的 MySQL 连接")]
    MysqlNotFound { name: String },

    /// MySQL 连接健康检查失败。
    #[cfg(feature = "mysql")]
    #[error("MySQL({name}) 健康检查失败: {message}")]
    MysqlHealthCheck {
        name: String,
        message: String,
        #[source]
        source: sqlx::Error,
    },

    // ── Redis 相关 ──
    /// Redis 连接打开失败。
    #[cfg(feature = "redis")]
    #[error("打开 Redis({url}) 失败: {message}")]
    RedisOpen {
        url: String,
        message: String,
        #[source]
        source: redis::RedisError,
    },

    /// Redis 连接建立失败。
    #[cfg(feature = "redis")]
    #[error("连接 Redis({url}) 失败: {message}")]
    RedisConnect {
        url: String,
        message: String,
        #[source]
        source: redis::RedisError,
    },

    /// Redis 命令执行失败。
    #[cfg(feature = "redis")]
    #[error("Redis 命令执行失败: {0}")]
    RedisCommand(#[from] redis::RedisError),

    /// Redis 连接名未找到。
    #[cfg(feature = "redis")]
    #[error("未找到名为 `{name}` 的 Redis 连接")]
    RedisNotFound { name: String },

    /// Redis 连接健康检查失败。
    #[cfg(feature = "redis")]
    #[error("Redis({name}) 健康检查失败: {message}")]
    RedisHealthCheck {
        name: String,
        message: String,
        #[source]
        source: redis::RedisError,
    },

    /// Redis 连接创建失败。
    #[cfg(feature = "redis")]
    #[error("创建 Redis 连接失败: {message}")]
    RedisConnectCreate { message: String },

    // ── 环境变量相关 ──
    /// 环境变量解析失败。
    #[error("环境变量 {key} 解析失败: {message}")]
    EnvParse { key: String, message: String },

    // ── Tracing 相关 ──
    /// tracing subscriber 已初始化，不可重复调用。
    #[cfg(feature = "tracing-init")]
    #[error("tracing subscriber 已初始化，不可重复调用")]
    TracingAlreadyInit,

    /// 无效的日志级别。
    #[cfg(feature = "tracing-init")]
    #[error("无效的日志级别: {0}")]
    TracingInvalidLevel(String),

    // ── HTTP 相关 ──
    /// HTTP 客户端创建失败。
    #[cfg(feature = "http")]
    #[error("创建 HTTP 客户端失败: {message}")]
    HttpClientCreate { message: String },

    /// HTTP 请求失败。
    #[cfg(feature = "http")]
    #[error("HTTP 请求失败: {source}")]
    HttpRequest {
        #[source]
        source: reqwest::Error,
    },
}

/// 用于 Config::build() 的 Result 类型别名。
pub type ConfigResult<T> = std::result::Result<T, Error>;

#[cfg(feature = "config-toml")]
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::ConfigParse {
            format: "TOML".into(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "config-yaml")]
impl From<yaml_serde::Error> for Error {
    fn from(e: yaml_serde::Error) -> Self {
        Error::ConfigParse {
            format: "YAML".into(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "config-json")]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::ConfigParse {
            format: "JSON".into(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "http")]
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpRequest { source: e }
    }
}
