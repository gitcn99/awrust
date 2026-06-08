//! # cc-core
//!
//! 公共核心库：分层配置系统 + MySQL / Redis 连接管理。
//!
//! ## 特性
//!
//! - **分层配置** — 支持 TOML / YAML / JSON 文件 → 环境变量 → 程序化覆盖
//! - **MySQL 连接池** — 多命名连接池管理，支持健康检查和优雅关闭
//! - **Redis 连接管理** — 多命名连接管理，支持自动重连和多路复用
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
//!     .build()
//!     .unwrap();
//! ```

pub mod error;

pub mod config;
pub use config::{Config, ConfigBuilder, MysqlConfig, RedisConfig};
pub use config::{IntoMysqlName, IntoRedisName, Validate};
pub use config::{MysqlConfigBuilder, RedisConfigBuilder};

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "redis")]
pub mod redis;

pub use error::{ConfigResult, Error};
