//! 优雅关闭示例：初始化 MySQL + Redis 连接，注册到 GracefulShutdown，
//! 然后等待 OS 信号（Ctrl+C / SIGTERM）自动逆序关闭所有资源。
//!
//! 运行：`cargo run --example graceful_shutdown`

use cc_core::mysql::MysqlPools;
use cc_core::redis::RedisManager;
use cc_core::{ConfigBuilder, GracefulShutdown};

#[tokio::main]
async fn main() -> cc_core::ConfigResult<()> {
    // 1. 加载配置（自动加载 config/config.<mode>.toml）
    let config = ConfigBuilder::new()?.build()?;

    // 初始化 tracing 日志
    cc_core::tracing::init_tracing(&config.tracing)?;

    // 2. 初始化连接池
    let mysql_pools = MysqlPools::from_config(&config).await?;
    let redis_manager = RedisManager::from_config(&config).await?;

    // 3. 健康检查
    mysql_pools.ping_all().await?;
    redis_manager.ping_all().await?;
    println!("所有连接就绪");

    // 4. 注册优雅关闭
    let mut shutdown = GracefulShutdown::new();

    // 注册自定义清理任务（先注册的最后关闭）
    shutdown.register("custom-cleanup", async {
        println!("执行自定义业务清理...");
        // 例如：取消后台任务、刷写缓冲区、释放文件锁等
    });

    // 注册 Redis 连接关闭
    shutdown.register_redis_manager(redis_manager);

    // 注册 MySQL 连接池关闭（最后注册，最先关闭）
    shutdown.register_mysql_pools(mysql_pools);

    // 5. 模拟业务运行
    println!("服务运行中，按 Ctrl+C 触发优雅关闭...");

    // 6. 等待 OS 信号，收到后按注册逆序执行：
    //    mysql-pools → redis-manager → custom-cleanup
    shutdown.wait_for_signal().await;

    println!("服务已安全退出");
    Ok(())
}
