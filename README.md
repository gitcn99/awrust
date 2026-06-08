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
│       │   └── redis.rs      # Redis 连接池管理
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

### 功能特性

- **分层配置** — 支持 TOML / YAML / JSON 文件 → 环境变量 → 程序化覆盖
- **MySQL 连接池** — 多命名连接池管理，支持健康检查和优雅关闭
- **Redis 连接池** — 多命名连接管理，支持自动重连

### 代码示例

| 示例                                                         | 说明             |
| ------------------------------------------------------------ | ---------------- |
| [basic_config.rs](crates/cc-core/examples/basic_config.rs)   | 基础配置加载     |
| [mysql_connect.rs](crates/cc-core/examples/mysql_connect.rs) | MySQL 连接池管理 |
| [redis_connect.rs](crates/cc-core/examples/redis_connect.rs) | Redis 连接池管理 |

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
