# cc-core 示例

## 配置

```bash
[ -f config/config.toml ] || cp config/config.toml.example config/config.toml
```

按需修改 `config/config.toml` 中的连接信息后运行示例。

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
