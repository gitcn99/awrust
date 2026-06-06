use cc_core::Config;

fn main() -> anyhow::Result<()> {
    let config = Config::load("config/config.toml")?;

    if let Some(mysql_config) = config.mysql("default") {
        println!("MySQL: {}:{}", mysql_config.host, mysql_config.port);
    }

    if let Some(redis_config) = config.redis("default") {
        println!("Redis: {}", redis_config.url);
    }

    Ok(())
}
