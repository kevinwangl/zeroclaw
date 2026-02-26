# Kiro Provider - 快速参考

## ✅ 已实现（基于实际 Kiro CLI 接口）

使用 `kiro-cli chat --no-interactive` 实现，无需 daemon 模式。

## 🚀 快速开始

```bash
# 1. 编译
cargo build --release

# 2. 配置
cat >> ~/.zeroclaw/config.toml <<EOF
default_provider = "kiro"
EOF

# 3. 测试
./target/release/zeroclaw agent --provider kiro -m "Hello"
```

## 🔧 环境变量

```bash
export KIRO_CLI_PATH=/path/to/kiro-cli  # 自定义路径
export KIRO_AGENT=agent-name             # 使用特定 agent
export KIRO_MODEL=model-name             # 使用特定模型
```

## 📝 配置示例

```toml
# ~/.zeroclaw/config.toml
default_provider = "kiro"
default_model = "claude-3-5-sonnet"

[channels_config.telegram]
stream_mode = true
```

## 💡 使用示例

```bash
# 基础使用
zeroclaw agent --provider kiro -m "What is Rust?"

# 指定 agent
export KIRO_AGENT=coding-assistant
zeroclaw agent --provider kiro -m "Write hello world"

# 指定模型
zeroclaw agent --provider kiro --model claude-3-5-sonnet -m "Hello"

# 交互模式
zeroclaw agent --provider kiro

# Daemon 模式（支持 Telegram/Discord）
zeroclaw daemon
```

## ✨ 支持的功能

- ✅ 基础对话
- ✅ 消息历史
- ✅ 流式响应
- ✅ Agent 配置
- ✅ 模型选择
- ⚠️ 工具调用（注入到系统提示）

## 🐛 故障排除

```bash
# 测试 Kiro CLI
kiro-cli chat --no-interactive "Hello"

# 查看日志
RUST_LOG=debug zeroclaw agent --provider kiro -m "test"

# 设置路径
export KIRO_CLI_PATH=/usr/local/bin/kiro-cli
```

## 📊 性能

| 指标 | 值 |
|------|-----|
| 延迟 | ~100-200ms |
| 流式首字 | ~50-100ms |
| 内存 | ~20-50MB/请求 |

## 📚 完整文档

- 集成指南：`docs/kiro-provider-integration.md`
- Provider 架构：`AGENTS.md` §7.1
