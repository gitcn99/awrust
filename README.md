# awrust

Rust 工作空间，包含 `cc-core` 核心公共库——提供命名模式配置系统 + MySQL / Redis 连接管理 + Tracing 日志初始化 + HTTP 客户端 + 优雅关闭。

## 项目结构

```
awrust/
├── crates/
│   └── cc-core/              # 核心公共库
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config/       # 命名模式配置系统
│       │   ├── mysql.rs      # MySQL 连接池管理
│       │   ├── redis.rs      # Redis 连接管理
│       │   ├── tracing.rs    # Tracing 日志初始化
│       │   ├── http.rs       # HTTP 客户端（基于 reqwest）
│       │   └── shutdown.rs   # 优雅关闭管理器
│       └── examples/         # 使用示例
├── config/                   # 配置文件
│   ├── config.dev.toml           # 开发环境配置（git ignored）
│   ├── config.online.toml        # 线上环境配置（git ignored）
│   └── config.example.toml       # 配置模板
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
[ -f config/config.dev.toml ] || cat > config/config.dev.toml << 'EOF'
[mysql.default]
host = "127.0.0.1"
port = 3306
user = "your_user"
password = "your_password"
database = "your_db"

[redis.default]
url = "redis://:password@127.0.0.1:6379"

[tracing]
level = "info"
format = "pretty"
EOF
```

默认按编译模式加载对应文件：debug → `config.dev.toml`，release → `config.online.toml`。
可通过 `CC_MODE=<name>` 环境变量切换到任意命名配置（加载 `config/config.<name>.toml`）。

### 功能特性

- **命名模式配置** — 每个环境独立配置文件（TOML / YAML / JSON），支持环境变量和程序化覆盖
- **MySQL 连接池** — 多命名连接池管理，支持健康检查和优雅关闭
- **Redis 连接管理** — 多命名连接管理，支持自动重连和多路复用
- **Tracing 初始化** — 从配置读取日志级别和输出格式（json/pretty），一键初始化
- **HTTP 客户端** — 基于 reqwest 的薄封装，支持 base_url、超时、默认请求头
- **优雅关闭** — 注册回调式关闭管理器，内置 MySQL / Redis 便捷注册 + OS 信号监听

### 代码示例

| 示例                                                                 | 说明                            |
| -------------------------------------------------------------------- | ------------------------------- |
| [basic_config.rs](crates/cc-core/examples/basic_config.rs)           | 基础配置加载                    |
| [mysql_connect.rs](crates/cc-core/examples/mysql_connect.rs)         | MySQL 连接池管理                |
| [redis_connect.rs](crates/cc-core/examples/redis_connect.rs)         | Redis 连接管理                  |
| [graceful_shutdown.rs](crates/cc-core/examples/graceful_shutdown.rs) | 优雅关闭（信号监听 + 回调注册） |
| [http_client.rs](crates/cc-core/examples/http_client.rs)             | HTTP 客户端使用                 |

## 开发

```bash
make env      # 初始化开发环境（安装 prettier、生成 config.dev.toml）
make fmt      # 格式化代码（prettier + cargo fmt）
make lint     # 格式化 + 检查 + clippy
make test     # 运行测试
make examples # 运行所有示例
make verify   # fmt + lint + test + examples
```

## 许可证

MIT
