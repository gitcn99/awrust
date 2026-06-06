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
async fn main() -> anyhow::Result<()> {
    // 分层配置：文件 → 环境变量 → 程序化覆盖
    let config = ConfigBuilder::new()
        .with_file("config/config.toml")?
        .with_env()?
        .build()?;

    let pools = MysqlPools::from_config(&config).await?;
    let pool = pools.require(MysqlName::Default)?;

    let version: (String,) = sqlx::query_as("SELECT VERSION()").fetch_one(pool).await?;
    println!("MySQL: {}", version.0);
    Ok(())
}
