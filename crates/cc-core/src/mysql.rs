//! MySQL 连接的初始化与多连接管理。

use std::collections::HashMap;

use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};

use crate::config::{Config, IntoMysqlName, MysqlConfig};

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
pub async fn connect(cfg: &MysqlConfig) -> anyhow::Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect_with(connect_options(cfg))
        .await?;
    Ok(pool)
}

/// 多个命名 MySQL 连接池的容器。
pub struct MysqlPools {
    pools: HashMap<String, MySqlPool>,
}

impl MysqlPools {
    /// 为配置里声明的每个 `[mysql.<名字>]` 建立连接池。
    pub async fn from_config(cfg: &Config) -> anyhow::Result<Self> {
        let mut pools = HashMap::new();
        for (name, mc) in &cfg.mysql {
            pools.insert(name.clone(), connect(mc).await?);
        }
        Ok(Self { pools })
    }

    /// 按名取连接池。
    pub fn get(&self, name: impl IntoMysqlName) -> Option<&MySqlPool> {
        self.pools.get(&name.into_name())
    }

    /// 按名取连接池，不存在时报错。
    pub fn require(&self, name: impl IntoMysqlName) -> anyhow::Result<&MySqlPool> {
        let name = name.into_name();
        self.pools
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("未找到名为 `{}` 的 MySQL 连接", name))
    }

    /// 获取默认连接池（名字为 "default"）。
    pub fn default(&self) -> anyhow::Result<&MySqlPool> {
        self.require("default")
    }
}
