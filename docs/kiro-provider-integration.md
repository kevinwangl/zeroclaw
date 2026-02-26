# 将 Kiro CLI 集成为 ZeroClaw Provider

本文档介绍如何将 Kiro CLI 作为 ZeroClaw 的 AI Provider 使用。

## 方案一：通过 `custom:` 前缀使用（最简单）

如果 Kiro CLI 提供了 OpenAI 兼容的 HTTP API 端点：

### 1. 启动 Kiro CLI HTTP 服务器

```bash
# 假设 Kiro CLI 提供了 HTTP 服务器模式
kiro-cli serve --port 8080
```

### 2. 配置 ZeroClaw

编辑 `~/.zeroclaw/config.toml`：

```toml
default_provider = "custom:http://localhost:8080"
default_model = "kiro-default-model"
api_key = "your-kiro-api-key"  # 如果需要认证
```

### 3. 使用

```bash
zeroclaw agent -m "Hello from ZeroClaw via Kiro!"
```

---

## 方案二：通过子进程调用 Kiro CLI（已实现）

这个方案通过子进程直接调用 `kiro-cli` 命令，无需 HTTP 服务器。

### 1. 确保 Kiro CLI 可执行

```bash
# 确认 kiro-cli 在 PATH 中
which kiro-cli

# 或设置自定义路径
export KIRO_CLI_PATH=/path/to/kiro-cli
```

### 2. 配置 ZeroClaw

编辑 `~/.zeroclaw/config.toml`：

```toml
default_provider = "kiro"
default_model = "kiro-model"  # 可选，通过 KIRO_MODEL 环境变量覆盖
```

或使用环境变量：

```bash
export KIRO_CLI_PATH=/usr/local/bin/kiro-cli
export KIRO_MODEL=your-preferred-model
```

### 3. 使用

```bash
# 基础使用
zeroclaw agent -m "Hello from ZeroClaw via Kiro!"

# 指定模型
zeroclaw agent --provider kiro --model custom-model -m "Test"

# 交互模式
zeroclaw agent --provider kiro
```

### 4. 工作原理

`KiroProvider` 实现：
- 将 ZeroClaw 的消息历史转换为 Kiro CLI 可理解的 prompt 格式
- 通过 `tokio::process::Command` 调用 `kiro-cli chat`
- 捕获标准输出作为响应
- 支持系统提示、用户消息和助手消息的格式化

### 5. 限制

- **不支持原生工具调用**：工具会被注入到系统提示中作为文本
- **不支持流式响应**：响应是一次性返回的
- **性能开销**：每次调用都会启动新的子进程

---

## 方案三：创建 Kiro HTTP 代理服务（推荐用于生产）

如果需要更好的性能和功能支持，可以创建一个独立的 HTTP 服务器来包装 Kiro CLI。

### 1. 创建 Kiro HTTP 服务器

```python
# kiro_server.py
from flask import Flask, request, jsonify
import subprocess
import json

app = Flask(__name__)

@app.route('/v1/chat/completions', methods=['POST'])
def chat_completions():
    data = request.json
    messages = data.get('messages', [])
    model = data.get('model', 'default')
    
    # 构建 prompt
    prompt = ""
    for msg in messages:
        role = msg['role']
        content = msg['content']
        if role == 'system':
            prompt += f"System: {content}\n\n"
        elif role == 'user':
            prompt += f"User: {content}\n\n"
        elif role == 'assistant':
            prompt += f"Assistant: {content}\n\n"
    
    prompt += "Assistant: "
    
    # 调用 kiro-cli
    result = subprocess.run(
        ['kiro-cli', 'chat', '--model', model],
        input=prompt,
        capture_output=True,
        text=True
    )
    
    response_text = result.stdout.strip()
    
    # 返回 OpenAI 兼容格式
    return jsonify({
        'id': 'kiro-' + str(hash(prompt)),
        'object': 'chat.completion',
        'created': int(time.time()),
        'model': model,
        'choices': [{
            'index': 0,
            'message': {
                'role': 'assistant',
                'content': response_text
            },
            'finish_reason': 'stop'
        }],
        'usage': {
            'prompt_tokens': len(prompt.split()),
            'completion_tokens': len(response_text.split()),
            'total_tokens': len(prompt.split()) + len(response_text.split())
        }
    })

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

### 2. 启动服务器

```bash
pip install flask
python kiro_server.py
```

### 3. 配置 ZeroClaw

```toml
default_provider = "custom:http://localhost:8080"
default_model = "kiro-model"
```

---

## 编译和测试

### 编译 ZeroClaw（包含 Kiro Provider）

```bash
cd /Users/sm4299/Downloads/bryan/zeroclaw
cargo build --release
```

### 测试 Kiro Provider

```bash
# 方法 1：直接测试
zeroclaw agent --provider kiro -m "What is Rust?"

# 方法 2：设置为默认 provider
zeroclaw onboard --provider kiro --force
zeroclaw agent -m "Hello"

# 方法 3：在 daemon 模式下使用
zeroclaw daemon
# 然后通过 Telegram/Discord 等 channel 发送消息
```

### 调试

```bash
# 启用详细日志
RUST_LOG=debug zeroclaw agent --provider kiro -m "test"

# 检查 provider 状态
zeroclaw status

# 测试 Kiro CLI 是否可用
kiro-cli --version
```

---

## 环境变量参考

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `KIRO_CLI_PATH` | Kiro CLI 可执行文件路径 | `kiro-cli` |
| `KIRO_MODEL` | 默认模型名称 | 无 |
| `RUST_LOG` | 日志级别 | `info` |

---

## 故障排除

### 问题：`kiro-cli: command not found`

**解决方案**：
```bash
# 设置完整路径
export KIRO_CLI_PATH=/usr/local/bin/kiro-cli

# 或添加到 PATH
export PATH=$PATH:/path/to/kiro/bin
```

### 问题：Kiro CLI 响应为空

**解决方案**：
- 检查 Kiro CLI 是否正常工作：`kiro-cli chat`
- 查看 ZeroClaw 日志：`RUST_LOG=debug zeroclaw agent --provider kiro -m "test"`
- 确认 prompt 格式是否正确

### 问题：性能较慢

**解决方案**：
- 使用方案三（HTTP 代理服务器）减少子进程启动开销
- 或使用方案一（如果 Kiro CLI 原生支持 HTTP 服务器）

---

## 下一步

1. **添加流式支持**：修改 `KiroProvider` 实现 `stream_chat` 方法
2. **添加工具调用支持**：如果 Kiro CLI 支持工具调用，实现 `supports_native_tools`
3. **优化性能**：考虑使用长驻进程或 HTTP 服务器模式
4. **添加配置选项**：在 `config.toml` 中添加 Kiro 特定配置

---

## 参考

- ZeroClaw Provider 架构：`docs/providers-reference.md`
- 添加自定义 Provider：`AGENTS.md` §7.1
- Kiro CLI 文档：（根据实际情况填写）
