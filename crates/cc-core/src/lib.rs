//! # cc-core
//!
//! 公共核心库：分层配置系统 + MySQL / Redis 连接管理 + Tracing 日志初始化 + HTTP 客户端 + 优雅关闭。
//!
//! ## 特性
//!
//! - **分层配置** — 支持 TOML / YAML / JSON 文件 → 环境变量 → 程序化覆盖
//! - **MySQL 连接池** — 多命名连接池管理，支持健康检查和优雅关闭
//! - **Redis 连接管理** — 多命名连接管理，支持自动重连和多路复用
//! - **Tracing 初始化** — 从配置读取日志级别和输出格式（json/pretty），一键初始化
//! - **HTTP 客户端** — 基于 reqwest 的薄封装，支持 base_url、超时、默认请求头
//! - **优雅关闭** — 注册回调式关闭管理器，内置 MySQL / Redis 便捷注册 + OS 信号监听
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use cc_core::ConfigBuilder;
//!
//! let config = ConfigBuilder::new()
//!     .with_mysql("default", |m| {
//!         m.host("127.0.0.1").user("root").password("pw").database("mydb")
//!     })
//!     .with_redis("cache", |r| r.url("redis://127.0.0.1:6379"))
//!     .with_tracing(|t| t.level("info").format("pretty"))
//!     .build()
//!     .unwrap();
//! ```

pub mod error;

pub mod config;
pub use config::{Config, ConfigBuilder, MysqlConfig, RedisConfig, TracingConfig};
pub use config::{IntoMysqlName, IntoRedisName, Validate};
pub use config::{MysqlConfigBuilder, RedisConfigBuilder, TracingConfigBuilder};

pub mod shutdown;
pub use shutdown::GracefulShutdown;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "tracing-init")]
pub mod tracing;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::{HttpClient, HttpClientBuilder};

pub use error::{ConfigResult, Error};
