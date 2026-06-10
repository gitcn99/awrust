use cc_core::ConfigBuilder;

fn main() -> cc_core::ConfigResult<()> {
    // 自动加载 config/config.<mode>.toml（按编译模式或 CC_MODE 选择）
    let config = ConfigBuilder::new()?.build()?;

    // 初始化 tracing 日志
    cc_core::tracing::init_tracing(&config.tracing)?;

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
