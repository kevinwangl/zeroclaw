# ZeroClaw 消息处理链路与上下文管理完整分析

> 版本: 2026-02-27 | 基于源码分析

---

## 一、全局架构概览

```
用户 (飞书/Telegram/Discord/...)
  │
  ▼
Channel (接收层)          ← src/channels/{lark,telegram,discord,...}.rs
  │
  ▼
Channel Dispatcher        ← src/channels/mod.rs::run_message_dispatch_loop
  │
  ▼
process_channel_message   ← src/channels/mod.rs:1508
  │
  ├─ 构建 System Prompt   ← build_channel_system_prompt / build_system_prompt_with_mode
  ├─ 加载对话历史          ← conversation_histories (内存 HashMap)
  ├─ 注入 Memory Context   ← build_memory_context (SQLite/Lucid)
  │
  ▼
run_tool_call_loop        ← src/agent/loop_.rs:2067
  │
  ├─ Multimodal 处理       ← src/multimodal.rs::prepare_messages_for_provider
  ├─ Provider.chat()       ← src/providers/traits.rs
  ├─ 解析 Tool Calls       ← parse_tool_calls / parse_structured_tool_calls
  ├─ 执行工具              ← Tool.execute()
  ├─ Tool Result → History ← history.push(...)
  └─ 循环直到无 tool_call 或达到上限
  │
  ▼
Channel.send()            ← 发送回复到用户
  │
  ▼
持久化历史                ← append_sender_turn (assistant response)
```

---

## 二、消息接收层 (Channel → Dispatcher)

### 2.1 Channel 监听

每个 Channel 实现 `Channel` trait (`src/channels/traits.rs`):

```rust
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, message: &SendMessage) -> anyhow::Result<()>;
    async fn listen(&self, tx: mpsc::Sender<ChannelMessage>) -> anyhow::Result<()>;
    async fn health_check(&self) -> bool;
}
```

以飞书为例 (`src/channels/lark.rs`):
- WebSocket 长连接接收消息
- 解析 protobuf 帧 → JSON event
- 过滤 `im.message.receive_v1` 事件
- 支持消息类型: `text`, `post`, `image`
- 生成 `ChannelMessage` 发送到 dispatcher

### 2.2 ChannelMessage 结构

```rust
pub struct ChannelMessage {
    pub id: String,           // UUID
    pub sender: String,       // 发送者标识 (chat_id)
    pub reply_target: String, // 回复目标 (chat_id)
    pub content: String,      // 消息文本内容
    pub channel: String,      // 频道名 ("feishu", "telegram")
    pub timestamp: u64,       // Unix 时间戳
    pub thread_ts: Option<String>, // 线程 ID (论坛/话题)
}
```

### 2.3 消息分发 (Dispatcher)

`run_message_dispatch_loop` (`src/channels/mod.rs`) 负责:
- 从 `mpsc::Receiver` 接收所有 channel 的消息
- 按 sender 维度管理并发 (中断旧请求)
- 限制并行处理数: `CHANNEL_PARALLELISM_PER_CHANNEL = 4`
- 最大在途消息: `8 ~ 64` (动态计算)
- 调用 `process_channel_message` 处理每条消息

---

## 三、消息处理核心 (process_channel_message)

**位置**: `src/channels/mod.rs:1508-2109`

### 3.1 处理流程

```
1. Hook: on_message_received (可修改/取消消息)
2. 运行时命令检测 (/new, /models, /provider 等)
3. Provider 初始化 (从缓存或工厂创建)
4. Memory 自动保存 (消息 ≥ 20 字符)
5. 构建对话历史
6. 构建 System Prompt
7. 调用 LLM (run_tool_call_loop)
8. 处理响应 (发送/错误处理)
9. 持久化历史
```

### 3.2 对话历史构建 (关键路径)

```
Step 1: 生成 history_key
  格式: "{channel}_{sender}" 或 "{channel}_{thread_ts}_{sender}"
  例: "feishu_oc_16364c267ab1fb2dc94b84f6be6f0093"

Step 2: 追加当前用户消息
  append_sender_turn(history_key, ChatMessage::user(msg.content))
  → 写入 conversation_histories HashMap
  → 上限: MAX_CHANNEL_HISTORY = 50 条

Step 3: 取出全部历史
  prior_turns = histories[history_key].clone()

Step 4: 归一化 (normalize_cached_channel_turns)
  → 确保 user/assistant 严格交替
  → 连续同角色消息合并

Step 5: Memory Context 注入 (仅首轮)
  → build_memory_context() 从 SQLite/Lucid 召回
  → 最多 4 条, 每条 ≤ 800 字符, 总计 ≤ 4000 字符
  → 注入到最后一条 user 消息前缀
```

### 3.3 对话历史存储

```rust
type ConversationHistoryMap = Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>;
```

| 属性 | 值 |
|------|-----|
| 存储位置 | 进程内存 (HashMap) |
| 持久化 | 无 (重启清除) |
| 上限 | 50 条/sender |
| 隔离粒度 | channel + sender (+ thread_ts) |
| 写入时机 | 用户消息到达时 + LLM 响应后 |
| 清除方式 | `/new` 命令 / 重启 / compact |

---

## 四、System Prompt 构建

**位置**: `src/channels/mod.rs:2253-2413`

### 4.1 组成结构

```
┌─────────────────────────────────────────────────┐
│ ## Tools                                         │
│   工具名称 + 描述列表                             │
│   例: "- screenshot: Capture a screenshot..."    │
│                                                  │
│ ## Hardware Access (条件)                         │
│   仅当有 gpio/arduino 工具时注入                  │
│                                                  │
│ ## Your Task                                     │
│   行为指令 (native-tools vs prompt-guided 不同)  │
│                                                  │
│ ## Safety                                        │
│   安全规则 (不泄露数据, 不执行破坏性命令等)       │
│                                                  │
│ ## Skills                                        │
│   技能 prompt (full 或 compact 模式)             │
│                                                  │
│ ## Workspace                                     │
│   工作目录路径                                    │
│                                                  │
│ ## Project Context                               │
│   Bootstrap 文件内容:                             │
│   IDENTITY.md / SOUL.md / USER.md / AGENTS.md   │
│   每个文件最多 20,000 字符                        │
│   或 AIEOS JSON identity                         │
│                                                  │
│ ## Current Date & Time                           │
│                                                  │
│ ## Runtime                                       │
│   Host / OS / Model 信息                         │
│                                                  │
│ ## Channel Capabilities                          │
│   频道行为指令                                    │
│                                                  │
│ + Channel context (reply_target, cron 投递信息)  │
└─────────────────────────────────────────────────┘
```

### 4.2 Tool Instructions 注入 (非 native-tools provider)

当 `provider.supports_native_tools() == false` 时 (`src/providers/traits.rs:335`):

```
Tool instructions 被转为文本注入到 system message:

## Tool Use Protocol

You have access to tools. To use a tool, emit:
<tool_call>
{"name": "tool_name", "arguments": {...}}
</tool_call>

Available tools:
- screenshot: {...schema...}
- shell: {...schema...}
- file_read: {...schema...}
...每个工具的完整 JSON schema
```

### 4.3 预估大小

| 组件 | 预估字符数 |
|------|-----------|
| Tools 列表 | 500 ~ 2,000 |
| Tool Instructions (prompt-guided) | 3,000 ~ 8,000 |
| Safety | ~300 |
| Skills | 0 ~ 5,000 |
| Bootstrap Files | 0 ~ 80,000 (4 × 20K) |
| Workspace/Runtime/Date | ~200 |
| Channel Context | ~300 |
| **总计** | **5K ~ 90K+** |

---

## 五、发送给 LLM 的完整 History

### 5.1 History 数组结构

```
history[0]: system  → System Prompt (含 tool instructions)
history[1]: user    → 第一条用户消息 (可能含 memory context)
history[2]: assistant → 第一条回复
...
history[N]: user    → 当前消息
```

### 5.2 Tool Loop 中的 History 增长

`run_tool_call_loop` (`src/agent/loop_.rs:2067`) 每次迭代:

```
迭代 1:
  → LLM 返回 <tool_call>screenshot</tool_call>
  → history.push(assistant: "<tool_call>...")
  → 执行 screenshot 工具
  → history.push(user: "[Tool results]\n<tool_result>...</tool_result>")

迭代 2:
  → LLM 返回最终文本
  → 返回文本作为响应
```

**关键**: Tool result 内容直接进入 history, 如果 tool 返回大量数据 (如 base64 图片), history 会急剧膨胀。

### 5.3 Multimodal 处理

`prepare_messages_for_provider` (`src/multimodal.rs`):

```
普通 provider:
  [IMAGE:/path/to/file] → data:image/png;base64,AAAA... (读取文件转 base64)

supports_raw_image_markers() == true 的 provider:
  [IMAGE:/path/to/file] → 保持原样 (provider 自行处理)
```

| 配置项 | 默认值 |
|--------|--------|
| max_images | 5 |
| max_image_size_mb | 10 |
| allow_remote_fetch | false |

---

## 六、Provider 调用链

### 6.1 通用流程

```
run_tool_call_loop
  │
  ├─ count_image_markers(history)  → 检查 vision 能力
  ├─ prepare_messages_for_provider → multimodal 处理
  │
  ▼
Provider.chat(ChatRequest)
  │
  ├─ supports_native_tools() == true:
  │    → 直接传 tool specs 给 API
  │    → API 返回结构化 tool_calls
  │
  └─ supports_native_tools() == false:
       → convert_tools() → ToolsPayload::PromptGuided
       → 注入 tool instructions 到 system message
       → chat_with_history(modified_messages)
       → 解析文本中的 <tool_call> 标签
```

### 6.2 Kiro Provider 特殊路径

```
KiroProvider.chat_with_history(messages)
  │
  ▼
messages_to_prompt(messages)  → 拼接为单个字符串
  │
  ▼
invoke_kiro(prompt)
  │
  ▼
kiro-cli chat --no-interactive  ← 通过 stdin 传入 prompt
  │
  ▼
kiro-cli 内部:
  ├─ 自己的 system prompt
  ├─ 自己的 context window 管理
  └─ 自己的工具系统 (read, shell 等)
```

**问题**: ZeroClaw 的 system prompt + tool instructions 被当作普通文本传给 kiro-cli, 而 kiro-cli 又加了自己的 system prompt, 导致双重 context 膨胀。

---

## 七、响应处理与历史持久化

### 7.1 成功响应

```
1. Hook: on_message_sending (可修改/取消)
2. sanitize_channel_response() → 清理残留 tool JSON
3. extract_tool_context_summary() → 提取工具使用摘要
4. append_sender_turn(history_key, assistant_response)
   → 只持久化最终文本, 不含 tool loop 中间过程
5. Channel.send(response) → 发送到用户
   → Lark: 检测 [IMAGE:path] marker → upload_image → 发送图片消息
   → Telegram: 检测 [IMAGE/DOCUMENT/VIDEO] markers → 发送媒体
```

### 7.2 错误处理

```
LLM 错误分类:
  │
  ├─ 取消 (newer message interrupt)
  │    → 不发送, 不持久化
  │
  ├─ Context Window Overflow
  │    → compact_sender_history()
  │    → 保留最近 12 条, 每条截断到 600 字符
  │    → 回复用户 "Context window exceeded..."
  │
  ├─ Tool Loop 超时
  │    → 回复错误信息
  │
  └─ 其他 LLM 错误
       → sanitize_api_error() → 脱敏
       → 回复用户错误信息
```

### 7.3 Context Overflow 检测

`is_context_window_overflow_error` (`src/channels/mod.rs:834`):

```rust
// 匹配关键词:
"exceeds the context window"
"context window of this model"
"maximum context length"
"context length exceeded"
"too many tokens"
"token limit exceeded"
"prompt is too long"
"input is too long"
```

**注意**: kiro-cli 的 overflow 消息 `"The context window has overflowed, summarizing..."` 不匹配以上任何关键词, 导致 compact 机制不会被触发。

---

## 八、关键常量汇总

| 常量 | 值 | 位置 | 说明 |
|------|-----|------|------|
| `MAX_CHANNEL_HISTORY` | 50 | channels/mod.rs | 每 sender 最大历史条数 |
| `BOOTSTRAP_MAX_CHARS` | 20,000 | channels/mod.rs | 每个 bootstrap 文件最大字符 |
| `MEMORY_CONTEXT_MAX_ENTRIES` | 4 | channels/mod.rs | Memory 召回最大条数 |
| `MEMORY_CONTEXT_ENTRY_MAX_CHARS` | 800 | channels/mod.rs | 每条 Memory 最大字符 |
| `MEMORY_CONTEXT_MAX_CHARS` | 4,000 | channels/mod.rs | Memory Context 总字符上限 |
| `CHANNEL_HISTORY_COMPACT_KEEP_MESSAGES` | 12 | channels/mod.rs | Compact 保留消息数 |
| `CHANNEL_HISTORY_COMPACT_CONTENT_CHARS` | 600 | channels/mod.rs | Compact 每条截断字符 |
| `CHANNEL_MESSAGE_TIMEOUT_SECS` | 300 | channels/mod.rs | 消息处理超时 (秒) |
| `DEFAULT_MAX_TOOL_ITERATIONS` | 10 | agent/loop_.rs | 工具调用最大迭代次数 |
| `AUTOSAVE_MIN_MESSAGE_CHARS` | 20 | channels/mod.rs | 自动保存最小消息长度 |

---

## 九、已知问题与风险点

### 9.1 Kiro Provider 双重 Context

**问题**: ZeroClaw 注入完整 system prompt + tool instructions → kiro-cli 再加自己的 → 双倍膨胀。

**影响**: 即使单轮对话也可能 overflow。

**建议**: kiro provider 的 `messages_to_prompt` 应只提取必要的 tool 定义, 跳过 bootstrap files / workspace / runtime 等冗余信息。

### 9.2 Tool Result 无大小限制

**问题**: `run_tool_call_loop` 中 tool result 直接 push 到 history, 无大小限制。

**影响**: 大文件读取、长命令输出等可能导致 history 膨胀。

**建议**: 在 push tool result 前截断到合理大小 (如 4K 字符)。

### 9.3 Overflow 检测不完整

**问题**: `is_context_window_overflow_error` 只匹配固定关键词, 不覆盖所有 provider 的错误格式。

**影响**: kiro provider 的 overflow 不触发 compact, 导致后续请求持续失败。

**建议**: 添加 kiro-cli 特有的 overflow 关键词, 或让 provider 实现 `is_context_overflow` 方法。

### 9.4 History 仅内存存储

**问题**: 对话历史不持久化, 重启即丢失。

**影响**: 长对话中重启 daemon 会丢失所有上下文。

**说明**: 这是设计选择, 非 bug。Memory 系统 (SQLite) 提供长期记忆。

---

## 十、数据流完整时序图

```
用户在飞书发送: "在当前界面截图压缩后回复我"
│
▼ [LarkChannel.listen_ws]
解析 WS 帧 → LarkEvent → ChannelMessage {
  sender: "oc_16364c...",
  content: "在当前界面截图压缩后回复我",
  channel: "feishu"
}
│
▼ [run_message_dispatch_loop]
mpsc::Receiver 收到消息 → spawn process_channel_message
│
▼ [process_channel_message]
│
├─ history_key = "feishu_oc_16364c..."
├─ append_sender_turn(user: "在当前界面截图压缩后回复我")
├─ prior_turns = [] (首次对话)
├─ build_memory_context() → "[Memory context]\n- ..."
├─ system_prompt = build_channel_system_prompt(...)
│    → ## Tools / ## Your Task / ## Safety / ## Project Context / ...
│
├─ history = [
│    system: "{system_prompt + tool_instructions}",  ← 可能 30K+
│    user: "[Memory context]\n在当前界面截图压缩后回复我"
│  ]
│
▼ [run_tool_call_loop] 迭代 1
│
├─ count_image_markers(history) → 0
├─ prepare_messages_for_provider(history) → 无变化
├─ provider.chat(history, tools) → 调用 kiro-cli
│    └─ kiro-cli 返回: <tool_call>{"name":"screenshot"}</tool_call>
│
├─ parse_tool_calls → [screenshot]
├─ execute screenshot tool → ToolResult {
│    output: "Screenshot captured (3970048 bytes).\n[IMAGE:/path/screenshot.png]"
│  }
│
├─ history.push(assistant: "<tool_call>...")
├─ history.push(user: "[Tool results]\n<tool_result>Screenshot captured...</tool_result>")
│
▼ [run_tool_call_loop] 迭代 2
│
├─ count_image_markers(history) → 1 (在 tool result 中)
├─ supports_vision() → true (kiro provider)
├─ supports_raw_image_markers() → true → 跳过 base64 转换
├─ provider.chat(history) → 调用 kiro-cli
│    └─ kiro-cli 读取 [IMAGE:path], 分析图片, 返回描述文本
│
├─ parse_tool_calls → [] (无 tool call)
├─ 返回最终文本: "这是你当前屏幕的截图...[IMAGE:/path/screenshot.png]"
│
▼ [process_channel_message 响应处理]
│
├─ sanitize_channel_response(response)
├─ append_sender_turn(assistant: response)
│
▼ [LarkChannel.send]
│
├─ extract_image_marker(response) → Some("/path/screenshot.png")
├─ upload_image(token, "/path/screenshot.png") → image_key
├─ 发送文本部分 (msg_type: "text")
├─ 发送图片部分 (msg_type: "image", image_key)
│
▼ 用户在飞书收到回复 (文本 + 图片)
```
