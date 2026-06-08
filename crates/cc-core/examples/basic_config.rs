use cc_core::ConfigBuilder;

fn main() -> cc_core::ConfigResult<()> {
    // 从文件加载（自动识别 TOML/YAML/JSON）
    let config = ConfigBuilder::from_file("config/config.toml")?.build()?;

    if let Some(mysql_config) = config.mysql("default") {
        println!("MySQL: {}:{}", mysql_config.host, mysql_config.port);
    }

    if let Some(redis_config) = config.redis("default") {
        println!("Redis: {}", redis_config.url);
    }

    // 打印所有连接名
    println!("MySQL 连接: {:?}", config.mysql_names().collect::<Vec<_>>());
    println!("Redis 连接: {:?}", config.redis_names().collect::<Vec<_>>());

    Ok(())
}
