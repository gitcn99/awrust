# cc-core 示例

## 配置

```bash
[ -f config/config.dev.toml ] || cp config/config.example.toml config/config.dev.toml
```

按需修改 `config/config.dev.toml` 中的连接信息后运行示例。

示例默认按编译模式加载对应文件：debug → `config.dev.toml`，release → `config.online.toml`。
可通过 `CC_MODE=<name>` 环境变量切换到任意命名配置（加载 `config/config.<name>.toml`）。

## 示例列表

### basic_config - 配置加载

```bash
cargo run --example basic_config
```

### mysql_connect - MySQL 连接

```bash
cargo run --example mysql_connect
```

### redis_connect - Redis 连接

```bash
cargo run --example redis_connect
```

### graceful_shutdown - 优雅关闭

监听 OS 信号（SIGTERM / SIGINT），按注册逆序执行关闭回调，支持 MySQL 连接池和 Redis 管理器的一键关闭。

```bash
cargo run --example graceful_shutdown
```

### http_client - HTTP 客户端

基于 reqwest 的封装客户端，支持 base_url 自动拼接、超时配置、默认请求头。

```bash
cargo run --example http_client
```
