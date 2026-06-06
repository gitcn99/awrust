//! 核心公共库：分层配置系统 + MySQL / Redis 连接管理。

pub mod config;
pub mod mysql;
pub mod redis;

pub use config::{Config, ConfigBuilder, MysqlConfig, RedisConfig};
pub use config::{IntoMysqlName, IntoRedisName, Validate};
pub use config::{MysqlConfigBuilder, RedisConfigBuilder};
