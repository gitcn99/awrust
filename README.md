# awrust

Rust 工作空间，包含 `cc-core` 核心公共库——提供分层配置系统 + MySQL / Redis 连接管理。

## 项目结构

```
awrust/
├── crates/
│   └── cc-core/              # 核心公共库
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config/       # 分层配置系统
│       │   ├── mysql.rs      # MySQL 连接池管理
│       │   └── redis.rs      # Redis 连接管理
│       └── examples/         # 使用示例
├── config/                   # 配置文件
│   ├── config.toml           # 实际配置（git ignored）
│   └── config.toml.example   # 配置模板
├── Cargo.toml                # 工作空间配置
└── Makefile                  # 开发命令
```

## 快速开始

### 添加依赖

```bash
cargo add cc-core
```

### 配置文件

```bash
[ -f config/config.toml ] || cat > config/config.toml << 'EOF'
[mysql.default]
host = "127.0.0.1"
port = 3306
user = "your_user"
password = "your_password"
database = "your_db"

[redis.default]
url = "redis://:password@127.0.0.1:6379"
EOF
```

### 分层配置

配置优先级从低到高：**TOML 文件 → 环境变量 → 程序化覆盖**。

```rust
use cc_core::ConfigBuilder;

fn main() -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .with_file("config/config.toml")?
        .with_env()?
        .build()?;
    Ok(())
}
```

环境变量格式：`CC_MYSQL_<NAME>_<FIELD>` / `CC_REDIS_<NAME>_<FIELD>`，可通过 `env_prefix()` 自定义前缀。

### MySQL 连接

```rust
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
```

### Redis 连接

```rust
use cc_core::{redis::RedisPools, ConfigBuilder, IntoRedisName};

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
    let config = ConfigBuilder::new()
        .with_file("config/config.toml")?
        .with_env()?
        .build()?;

    let pools = RedisPools::from_config(&config).await?;
    let mut conn = pools.require(RedisName::Default)?;

    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    println!("PING: {}", pong);
    Ok(())
}
```

## 开发

```bash
make env      # 初始化开发环境（安装 prettier、生成 config.toml）
make fmt      # 格式化代码（prettier + cargo fmt）
make lint     # 格式化 + 检查 + clippy
make test     # 运行测试
make examples # 运行所有示例
make verify   # fmt + lint + test + examples
```

## 许可证

MIT
