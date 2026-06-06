//! 基本配置加载示例
//!
//! 演示如何从 TOML 文件加载配置并访问配置项。

use std::path::Path;

use cc_core::Config;

fn main() -> anyhow::Result<()> {
    // 加载配置文件
    let config_path = Path::new("config/config.toml");

    // 如果配置文件不存在，使用示例配置
    let config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        println!("配置文件不存在，使用内嵌示例配置");

        let toml_content = r#"
            [mysql.default]
            host = "localhost"
            port = 3306
            user = "root"
            password = "password"
            database = "test_db"
            max_connections = 5
            ssl_mode = "preferred"
            
            [redis.default]
            url = "redis://127.0.0.1:6379"
        "#;

        toml::from_str(toml_content)?
    };

    // 访问 MySQL 配置
    if let Some(mysql_config) = config.mysql("default") {
        println!("MySQL 配置:");
        println!("  主机: {}", mysql_config.host);
        println!("  端口: {}", mysql_config.port);
        println!("  用户: {}", mysql_config.user);
        println!("  数据库: {}", mysql_config.database);
        println!("  最大连接数: {}", mysql_config.max_connections);
        println!("  SSL 模式: {}", mysql_config.ssl_mode);
    }

    // 访问 Redis 配置
    if let Some(redis_config) = config.redis("default") {
        println!("\nRedis 配置:");
        println!("  URL: {}", redis_config.url);
    }

    // 列出所有配置的连接
    println!(
        "\n所有 MySQL 连接: {:?}",
        config.mysql.keys().collect::<Vec<_>>()
    );
    println!(
        "所有 Redis 连接: {:?}",
        config.redis.keys().collect::<Vec<_>>()
    );

    Ok(())
}
