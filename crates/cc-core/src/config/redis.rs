use std::collections::HashMap;

use serde::Deserialize;

use super::Validate;

// ──────────────────────────────────────────────
// Redis 配置
// ──────────────────────────────────────────────

/// 单个 Redis 连接的配置。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RedisConfig {
    /// 形如 `redis://[:password@]host:port[/db]` 的连接串
    pub url: String,
}

impl Validate for RedisConfig {
    fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_empty() {
            anyhow::bail!("Redis url 不能为空");
        }
        if !self.url.starts_with("redis://") && !self.url.starts_with("rediss://") {
            anyhow::bail!(
                "Redis url 格式无效: `{}`，需以 `redis://` 或 `rediss://` 开头",
                self.url
            );
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────
// Redis 子构建器
// ──────────────────────────────────────────────

/// Redis 单连接构建器。
pub struct RedisConfigBuilder(pub(crate) RedisConfig);

impl RedisConfigBuilder {
    pub fn url(mut self, v: impl Into<String>) -> Self {
        self.0.url = v.into();
        self
    }
}

// ──────────────────────────────────────────────
// 环境变量解析
// ──────────────────────────────────────────────

pub(crate) fn collect_env_redis(
    prefix: &str,
    existing: &HashMap<String, RedisConfig>,
) -> anyhow::Result<HashMap<String, RedisConfig>> {
    let mut result = HashMap::new();
    let pfx_upper = prefix.to_uppercase();

    for (key, val) in std::env::vars() {
        let upper = key.to_uppercase();
        let rest = match upper.strip_prefix(&format!("{pfx_upper}_REDIS_")) {
            Some(r) => r,
            None => continue,
        };
        let (name, field) = match rest.rsplit_once('_') {
            Some((n, f)) => (n.to_lowercase(), f),
            None => continue,
        };

        if field == "URL" {
            let entry = result
                .entry(name.clone())
                .or_insert_with(|| existing.get(&name).cloned().unwrap_or_default());
            entry.url = val;
        }
    }
    Ok(result)
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::ConfigBuilder;

    #[test]
    fn validation_rejects_bad_redis_url() {
        let result = ConfigBuilder::new()
            .with_redis("default", |r| r.url("http://wrong"))
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("redis://"));
    }
}
