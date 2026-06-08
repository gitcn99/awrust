use cc_core::{mysql::MysqlPools, ConfigBuilder, IntoMysqlName};

enum MysqlName {
    Default,
}

impl IntoMysqlName for MysqlName {
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

    let pools = MysqlPools::from_config(&config).await?;
    let pool = pools.require(MysqlName::Default)?;

    let version: (String,) = sqlx::query_as("SELECT VERSION()").fetch_one(pool).await?;
    println!("MySQL: {}", version.0);

    pools.ping_all().await?;
    println!("所有 MySQL 连接健康检查通过");

    Ok(())
}
