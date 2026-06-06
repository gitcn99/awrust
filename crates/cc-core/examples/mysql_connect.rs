//! MySQL 连接示例
//!
//! 演示如何建立 MySQL 连接并执行查询。
//! 注意：需要实际的 MySQL 服务器才能运行此示例。

use std::path::Path;

use cc_core::{mysql::MysqlPools, Config, IntoMysqlName};

// 定义连接名枚举，获得编译时检查
#[allow(dead_code)]
enum MysqlName {
    Default,
    OrderDb,
}

impl IntoMysqlName for MysqlName {
    fn into_name(self) -> String {
        match self {
            Self::Default => "default".into(),
            Self::OrderDb => "order_db".into(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载配置
    let config_path = Path::new("config/config.toml");
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
        "#;

        toml::from_str(toml_content)?
    };

    // MysqlPools（推荐方式）
    let mysql_pools = MysqlPools::from_config(&config).await?;

    // 使用枚举获取连接，编译时检查
    let pool = mysql_pools.require(MysqlName::Default)?;
    println!("从 MysqlPools 获取连接成功！");

    // 执行简单查询
    let result: (i64,) = sqlx::query_as("SELECT 1").fetch_one(pool).await?;
    println!("查询结果: {}", result.0);

    // 查询数据库版本
    let version: (String,) = sqlx::query_as("SELECT VERSION()").fetch_one(pool).await?;
    println!("MySQL 版本: {}", version.0);

    // 关闭连接池
    pool.close().await;
    println!("连接已关闭");

    Ok(())
}
