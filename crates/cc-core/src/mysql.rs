//! MySQL 连接的初始化与多连接管理。

use std::collections::HashMap;
use std::time::Duration;

use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};

use crate::config::{Config, IntoMysqlName, MysqlConfig};
use crate::error::{ConfigResult, Error};

/// 把配置里的字符串 ssl_mode 映射到 sqlx 的枚举（无法识别时回退 Preferred）。
pub fn ssl_mode_from_str(s: &str) -> MySqlSslMode {
    match s.trim().to_ascii_lowercase().as_str() {
        "disabled" | "disable" | "off" => MySqlSslMode::Disabled,
        "required" | "require" => MySqlSslMode::Required,
        "verify-ca" | "verify_ca" => MySqlSslMode::VerifyCa,
        "verify-identity" | "verify_identity" => MySqlSslMode::VerifyIdentity,
        _ => MySqlSslMode::Preferred,
    }
}

/// 根据配置构造连接选项。
pub fn connect_options(cfg: &MysqlConfig) -> MySqlConnectOptions {
    let mut opts = MySqlConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.user)
        .password(&cfg.password)
        .ssl_mode(ssl_mode_from_str(&cfg.ssl_mode));

    if cfg.disable_sql_mode {
        opts = opts.no_engine_substitution(false).pipes_as_concat(false);
    }

    if !cfg.database.is_empty() {
        opts = opts.database(&cfg.database);
    }
    opts
}

/// 用单个配置建立连接池。
pub async fn connect(cfg: &MysqlConfig) -> ConfigResult<MySqlPool> {
    tracing::info!(host = %cfg.host, port = cfg.port, database = %cfg.database, "建立 MySQL 连接");
    let pool_options = MySqlPoolOptions::new()
        .max_connections(cfg.max_connections)
        .acquire_timeout(Duration::from_secs(cfg.acquire_timeout.into()))
        .idle_timeout(Duration::from_secs(cfg.idle_timeout.into()));

    let pool = pool_options
        .connect_with(connect_options(cfg))
        .await
        .map_err(|e| Error::MysqlConnect {
            host: cfg.host.clone(),
            port: cfg.port,
            source: e,
        })?;
    tracing::info!(host = %cfg.host, "MySQL 连接建立成功");
    Ok(pool)
}

/// MySQL 连接池的统计信息。
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// 当前活跃连接数
    pub active: usize,
    /// 当前空闲连接数
    pub idle: usize,
}

/// 多个命名 MySQL 连接池的容器。
#[derive(Debug)]
pub struct MysqlPools {
    pools: HashMap<String, MySqlPool>,
}

impl MysqlPools {
    /// 为配置里声明的每个 `[mysql.<名字>]` 建立连接池。
    pub async fn from_config(cfg: &Config) -> ConfigResult<Self> {
        let mut pools = HashMap::new();
        for (name, mc) in &cfg.mysql {
            tracing::info!(name = %name, "初始化 MySQL 连接池");
            pools.insert(name.clone(), connect(mc).await?);
        }
        tracing::info!(count = pools.len(), "所有 MySQL 连接池初始化完成");
        Ok(Self { pools })
    }

    /// 按名取连接池。
    pub fn get(&self, name: impl IntoMysqlName) -> Option<&MySqlPool> {
        self.pools.get(&name.into_name())
    }

    /// 按名取连接池，不存在时报错。
    pub fn require(&self, name: impl IntoMysqlName) -> ConfigResult<&MySqlPool> {
        let name = name.into_name();
        self.pools
            .get(&name)
            .ok_or_else(|| Error::MysqlNotFound { name })
    }

    /// 获取默认连接池（名字为 "default"）。
    pub fn default(&self) -> ConfigResult<&MySqlPool> {
        self.require("default")
    }

    /// 健康检查：对指定连接执行 `SELECT 1`。
    pub async fn ping(&self, name: impl IntoMysqlName) -> ConfigResult<()> {
        let name_str = name.into_name();
        let pool = self.require(&name_str)?;
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
            .map_err(|e| Error::MysqlHealthCheck {
                name: name_str,
                message: e.to_string(),
                source: e,
            })?;
        Ok(())
    }

    /// 健康检查：检查所有连接。
    pub async fn ping_all(&self) -> ConfigResult<()> {
        for name in self.pools.keys() {
            self.ping(name.as_str()).await?;
        }
        Ok(())
    }

    /// 获取指定连接池的统计信息。
    pub fn stats(&self, name: impl IntoMysqlName) -> Option<PoolStats> {
        let pool = self.pools.get(&name.into_name())?;
        let size = pool.size() as usize;
        let idle = pool.num_idle();
        Some(PoolStats {
            active: size.saturating_sub(idle),
            idle,
        })
    }

    /// 获取所有连接池名称。
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.pools.keys().map(String::as_str)
    }

    /// 关闭所有连接池。
    pub async fn shutdown(self) {
        tracing::info!("关闭所有 MySQL 连接池");
        for (name, pool) in &self.pools {
            tracing::debug!(name = %name, "关闭 MySQL 连接池");
            pool.close().await;
        }
        tracing::info!("所有 MySQL 连接池已关闭");
    }
}
