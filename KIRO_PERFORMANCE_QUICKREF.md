# Kiro Provider 性能优化 - 快速参考

## 🚀 核心优化

| 特性 | 性能提升 | 启用方式 |
|------|----------|----------|
| **Daemon 模式** | 延迟降低 95% | `export KIRO_USE_DAEMON=true` |
| **流式响应** | 首字延迟降低 90% | 自动启用（需 daemon） |
| **自动故障恢复** | 零停机 | 默认启用 |

## ⚡ 快速启用

```bash
# 1. 启用 daemon 模式
export KIRO_USE_DAEMON=true

# 2. 编译 ZeroClaw
cargo build --release

# 3. 测试
./target/release/zeroclaw agent --provider kiro -m "Hello"

# 4. 运行基准测试
./benchmark_kiro.sh
```

## 📊 性能数据

```
模式          首次调用    后续调用    并发能力
─────────────────────────────────────────────
Oneshot       200ms      150ms       低
Daemon (冷)   180ms      10ms        高
Daemon (热)   10ms       5ms         高
```

## 🔧 环境变量

```bash
# 必需
export KIRO_USE_DAEMON=true          # 启用 daemon 模式

# 可选
export KIRO_CLI_PATH=/path/to/kiro   # 自定义路径
export KIRO_MODEL=model-name         # 默认模型
export RUST_LOG=debug                # 调试日志
```

## 📝 配置文件

```toml
# ~/.zeroclaw/config.toml
default_provider = "kiro"
default_model = "kiro-model"

[channels_config.telegram]
stream_mode = true
draft_update_interval_ms = 500
```

## 🐛 故障排除

```bash
# 检查 daemon 是否运行
ps aux | grep "kiro-cli daemon"

# 杀死僵死的 daemon
pkill -f "kiro-cli daemon"

# 查看详细日志
RUST_LOG=zeroclaw::providers::kiro=debug zeroclaw agent --provider kiro -m "test"

# 回退到 oneshot 模式
export KIRO_USE_DAEMON=false
```

## 📚 完整文档

- 性能优化详解：`docs/kiro-provider-performance.md`
- 集成指南：`docs/kiro-provider-integration.md`
- 基准测试：`./benchmark_kiro.sh`
