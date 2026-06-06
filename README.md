# awrust

Rust 工作空间，包含 `cc-core` 核心公共库。

## 项目结构

```
awrust/
├── crates/
│   └── cc-core/          # 核心公共库
│       ├── src/          # 库源码
│       └── examples/     # 使用示例
├── config/               # 配置文件
└── Cargo.toml            # 工作空间配置
```

## 快速开始

### 添加依赖

```bash
cargo add cc-core
```

### 配置文件

新建 `config/config.toml`，填入实际连接信息：

```toml
[mysql.default]
host = "127.0.0.1"
port = 3306
user = "your_user"
password = "your_password"
database = "your_db"

[redis.default]
url = "redis://127.0.0.1:6379"
```

### MySQL 连接

```rust
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
```

### Redis 连接

```rust
use cc_core::{redis::RedisPools, Config, IntoRedisName};

enum RedisName {
    Default,
}

impl IntoRedisName for RedisName {
    fn into_name(self) -> String {
        match self {
            Self::Default => "default".into(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load("config/config.toml")?;
    let pools = RedisPools::from_config(&config).await?;
    let mut conn = pools.require(RedisName::Default)?;

    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    println!("PING: {}", pong);
    Ok(())
}
```
