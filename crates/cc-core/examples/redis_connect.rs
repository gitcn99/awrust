//! Redis 连接示例
//!
//! 演示如何建立 Redis 连接并执行基本操作。
//! 注意：需要实际的 Redis 服务器才能运行此示例。

use std::path::Path;

use cc_core::{redis::RedisPools, Config, IntoRedisName};

// 定义连接名枚举，获得编译时检查
#[allow(dead_code)]
enum RedisName {
    Default,
    Session,
}

impl IntoRedisName for RedisName {
    fn into_name(self) -> String {
        match self {
            Self::Default => "default".into(),
            Self::Session => "session".into(),
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
            [redis.default]
            url = "redis://127.0.0.1:6379"
        "#;

        toml::from_str(toml_content)?
    };

    // 建立 Redis 连接池
    println!("正在连接 Redis 服务器...");
    let redis_pools = RedisPools::from_config(&config).await?;

    // 获取默认连接（使用枚举，编译时检查）
    let mut conn = redis_pools.require(RedisName::Default)?;

    println!("Redis 连接成功！");

    // 执行 PING 命令
    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;

    println!("PING 响应: {}", pong);

    // 设置键值
    let _: () = redis::cmd("SET")
        .arg("greeting")
        .arg("Hello from cc-core!")
        .query_async(&mut conn)
        .await?;

    println!("已设置键 'greeting'");

    // 获取键值
    let greeting: String = redis::cmd("GET")
        .arg("greeting")
        .query_async(&mut conn)
        .await?;

    println!("获取键 'greeting': {}", greeting);

    // 删除键
    let _: i32 = redis::cmd("DEL")
        .arg("greeting")
        .query_async(&mut conn)
        .await?;

    println!("已删除键 'greeting'");

    // 获取 Redis 信息
    let info: String = redis::cmd("INFO")
        .arg("server")
        .query_async(&mut conn)
        .await?;

    println!(
        "Redis 服务器信息（前200字符）: {}",
        &info[..200.min(info.len())]
    );

    Ok(())
}
