use cc_core::{redis::RedisPools, Config, IntoRedisName};

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
    let config = Config::load("config/config.toml")?;
    let pools = RedisPools::from_config(&config).await?;
    let mut conn = pools.require(RedisName::Default)?;

    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    println!("PING: {}", pong);
    Ok(())
}
