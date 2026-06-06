//! 核心公共库：集中维护 MySQL / Redis 的初始化与连接，配置来自 config/config.toml 文件。

pub mod config;
pub mod mysql;
pub mod redis;

pub use config::{Config, MysqlConfig, RedisConfig};
pub use config::{IntoMysqlName, IntoRedisName};
