//! Redis 连接的初始化与多连接管理。
//!
//! 用 `::redis` 显式引用外部 crate，以区别于本模块 `crate::redis`。

use std::collections::HashMap;

use ::redis::aio::ConnectionManager;

use crate::config::{Config, IntoRedisName, RedisConfig};

/// 用单个配置建立一个带自动重连的连接管理器（可廉价 clone 共享）。
pub async fn connect(cfg: &RedisConfig) -> anyhow::Result<ConnectionManager> {
    let client = ::redis::Client::open(cfg.url.as_str())
        .map_err(|e| anyhow::anyhow!("打开 Redis({}) 失败: {}", cfg.url, e))?;
    let mgr = ConnectionManager::new(client)
        .await
        .map_err(|e| anyhow::anyhow!("连接 Redis({}) 失败: {}", cfg.url, e))?;
    Ok(mgr)
}

/// 多个命名 Redis 连接的容器。
pub struct RedisPools {
    conns: HashMap<String, ConnectionManager>,
}

impl RedisPools {
    /// 为配置里声明的每个 `[redis.<名字>]` 建立连接。
    pub async fn from_config(cfg: &Config) -> anyhow::Result<Self> {
        let mut conns = HashMap::new();
        for (name, rc) in &cfg.redis {
            conns.insert(name.clone(), connect(rc).await?);
        }
        Ok(Self { conns })
    }

    /// 按名取连接（克隆出一份句柄，底层连接共享）。
    pub fn get(&self, name: impl IntoRedisName) -> Option<ConnectionManager> {
        self.conns.get(&name.into_name()).cloned()
    }

    /// 按名取连接，不存在时报错。
    pub fn require(&self, name: impl IntoRedisName) -> anyhow::Result<ConnectionManager> {
        let name = name.into_name();
        self.conns
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("未找到名为 `{}` 的 Redis 连接", name))
    }

    /// 获取默认连接（名字为 "default"）。
    pub fn default(&self) -> anyhow::Result<ConnectionManager> {
        self.require("default")
    }
}
