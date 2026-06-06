use cc_core::ConfigBuilder;

fn main() -> anyhow::Result<()> {
    // 从文件加载
    let config = ConfigBuilder::from_file("config/config.toml")?.build()?;

    if let Some(mysql_config) = config.mysql("default") {
        println!("MySQL: {}:{}", mysql_config.host, mysql_config.port);
    }

    if let Some(redis_config) = config.redis("default") {
        println!("Redis: {}", redis_config.url);
    }

    Ok(())
}
