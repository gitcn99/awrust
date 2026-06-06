use cc_core::{redis::RedisPools, ConfigBuilder, IntoRedisName};

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
async fn main() -> anyhow::Result<()> {
    // 分层配置：文件 → 环境变量 → 程序化覆盖
    let config = ConfigBuilder::new()
        .with_file("config/config.toml")?
        .with_env()?
        .build()?;

    let pools = RedisPools::from_config(&config).await?;
    let mut conn = pools.require(RedisName::Default)?;

    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    println!("PING: {}", pong);
    Ok(())
}
