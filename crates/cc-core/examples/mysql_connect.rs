use cc_core::{mysql::MysqlPools, Config, IntoMysqlName};

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
async fn main() -> anyhow::Result<()> {
    let config = Config::load("config/config.toml")?;
    let pools = MysqlPools::from_config(&config).await?;
    let pool = pools.require(MysqlName::Default)?;

    let version: (String,) = sqlx::query_as("SELECT VERSION()").fetch_one(pool).await?;
    println!("MySQL: {}", version.0);
    Ok(())
}
