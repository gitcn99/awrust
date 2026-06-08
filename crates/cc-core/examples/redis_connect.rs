use cc_core::{redis::RedisManager, ConfigBuilder, IntoRedisName};
use redis::AsyncTypedCommands;

enum RedisName {
    Default,
}

impl IntoRedisName for RedisName {
    fn into_name(self) -> String {
        match self {
            Self::Default => "default".into(),
        }
    }
}

#[tokio::main]
async fn main() -> cc_core::ConfigResult<()> {
    let config = ConfigBuilder::new()
        .with_file("config/config.toml")?
        .with_env()?
        .build()?;

    let manager = RedisManager::from_config(&config).await?;
    let conn = manager.require(RedisName::Default)?;

    let mut cm = conn.get_connection();
    let pong: String = cm.ping().await?;
    println!("PING: {pong}");

    // 批量健康检查
    manager.ping_all().await?;
    println!("所有 Redis 连接健康检查通过");

    Ok(())
}
