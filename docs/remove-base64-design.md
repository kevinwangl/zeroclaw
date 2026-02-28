# 移除不必要的 Base64 转换 - 设计方案

## 🎯 目标

**完全移除不必要的 base64 转换，优先使用本地文件路径。**

## 📊 当前问题

### 问题 1：过度转换
```rust
// 当前逻辑（src/agent/loop_.rs:2108-2128）
let image_marker_count = multimodal::count_image_markers(history);

if image_marker_count > 0 && !provider.supports_vision() {
    return Err(...);  // ❌ 直接报错
}

let prepared_messages = if provider.supports_raw_image_markers() {
    // 保持原始标记
} else {
    multimodal::prepare_messages_for_provider(...)  // ❌ 转换所有图片为 base64
};
```

**问题**：
- 即使 provider 支持 vision，也会转换所有图片为 base64
- 即使图片只是工具输出（不需要 LLM 分析），也会转换
- 浪费 CPU、内存、token

### 问题 2：错误的假设

系统假设：**所有图片标记都需要发送给 LLM**

实际情况：
- ✅ 用户上传图片 → 需要 LLM 分析 → 需要 base64
- ❌ 工具生成图片 → 只需发送给用户 → **不需要** base64

## 🔧 解决方案

### 方案 A：区分图片来源（推荐）

```rust
// 新增：区分用户上传的图片 vs 工具生成的图片
enum ImageSource {
    UserUpload,      // 用户上传，需要 LLM 分析
    ToolOutput,      // 工具输出，只需发送给用户
}

// 在 ChatMessage 中标记图片来源
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub image_source: Option<ImageSource>,  // 新增
}
```

**工作流程**：
```
1. 用户消息包含图片 → image_source = UserUpload
   → 如果 provider.supports_vision() → 转换 base64
   → 否则 → 报错

2. 工具结果包含图片 → image_source = ToolOutput
   → 保持本地路径，不转换
   → LLM 看到 [IMAGE:/tmp/screenshot.png]
   → LLM 回复包含原始路径
   → Channel 根据能力处理
```

### 方案 B：延迟转换（更简单）

```rust
// 修改逻辑：只在必要时转换
let prepared_messages = if provider.supports_vision() && contains_user_uploaded_images(history) {
    // 只转换用户上传的图片
    multimodal::prepare_messages_for_provider(history, multimodal_config).await?
} else {
    // 保持所有图片为本地路径
    PreparedMessages {
        contains_images: false,  // 告诉 provider 不要期待图片
        messages: strip_image_markers_from_tool_results(history),
    }
};
```

**关键函数**：
```rust
fn strip_image_markers_from_tool_results(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    messages.iter().map(|msg| {
        if msg.role == "tool" {
            // 工具结果中的图片标记保持不变，但告诉 provider 忽略它们
            ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            }
        } else {
            msg.clone()
        }
    }).collect()
}
```

### 方案 C：完全移除 base64（激进）

```rust
// 1. 移除 multimodal::prepare_messages_for_provider() 调用
// 2. 所有图片保持本地路径
// 3. Vision provider 通过本地 HTTP 服务器访问图片

let prepared_messages = PreparedMessages {
    contains_images: multimodal::contains_image_markers(history),
    messages: history.to_vec(),
};

// 如果 provider 需要 vision，启动临时 HTTP 服务器
if provider.supports_vision() && prepared_messages.contains_images {
    let server = start_local_file_server()?;
    let messages_with_urls = convert_paths_to_localhost_urls(history, &server);
    // 发送给 provider
}
```

## 📝 实施计划

### 阶段 1：快速修复（立即）

**目标**：移除工具输出图片的 base64 转换

**修改**：`src/agent/loop_.rs`

```rust
// 修改前
let image_marker_count = multimodal::count_image_markers(history);
if image_marker_count > 0 && !provider.supports_vision() {
    return Err(...);
}

// 修改后
let user_image_count = count_user_uploaded_images(history);
if user_image_count > 0 && !provider.supports_vision() {
    return Err(...);  // 只检查用户上传的图片
}

// 工具输出的图片不转换
let prepared_messages = if provider.supports_vision() && user_image_count > 0 {
    multimodal::prepare_messages_for_provider(history, multimodal_config).await?
} else {
    PreparedMessages {
        contains_images: false,
        messages: history.to_vec(),
    }
};
```

**新增函数**：
```rust
fn count_user_uploaded_images(messages: &[ChatMessage]) -> usize {
    messages.iter()
        .filter(|msg| msg.role == "user")
        .map(|msg| multimodal::count_image_markers_in_text(&msg.content))
        .sum()
}
```

### 阶段 2：Channel 本地能力（短期）

**目标**：让 channel 利用本地文件路径

**修改**：各个 channel 的 `send()` 方法

```rust
// CLI Channel
async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
    let (text, attachments) = parse_attachment_markers(&message.content);
    
    println!("{}", text);
    
    for attachment in &attachments {
        if is_local_path(&attachment.target) {
            println!("📎 {}: {}", attachment.kind.marker_name(), attachment.target);
            
            // 复制到剪贴板
            if let Err(e) = copy_to_clipboard(&attachment.target) {
                println!("⚠️  无法复制到剪贴板: {}", e);
            } else {
                println!("✅ 文件路径已复制到剪贴板");
            }
        }
    }
    
    Ok(())
}
```

### 阶段 3：移除 multimodal 模块（长期）

**目标**：完全移除 base64 转换逻辑

**删除**：
- `src/multimodal.rs` 中的 `prepare_messages_for_provider()`
- `src/multimodal.rs` 中的 `normalize_local_image()`
- 所有 base64 编码逻辑

**保留**：
- `parse_image_markers()` - 解析标记
- `count_image_markers()` - 计数

## 🎯 预期效果

### 性能改善

| 场景 | 当前 | 改进后 |
|------|------|--------|
| screenshot 工具 | 3.9MB → 5.2MB base64 | 3.9MB 文件路径（~50 字节） |
| CPU 使用 | Base64 编码（~100ms） | 0ms |
| 内存使用 | 5.2MB 字符串 | 50 字节字符串 |
| LLM token | ~7000 tokens | ~10 tokens |

### 用户体验改善

| Channel | 当前 | 改进后 |
|---------|------|--------|
| CLI | 显示 base64 字符串 | 显示文件路径 + 复制到剪贴板 |
| DingTalk | 显示 base64 字符串 | 显示友好提示 + 文件路径 |
| Telegram | 上传文件（从 base64） | 上传文件（从路径） |
| Discord | 上传文件（从 base64） | 上传文件（从路径） |

## ⚠️ 注意事项

### Vision Provider 的限制

某些 vision provider **必须**使用 base64 或 URL：
- OpenAI GPT-4V
- Anthropic Claude 3
- Google Gemini

**解决方案**：
1. 只在用户明确要求分析图片时转换
2. 或者实现本地 HTTP 服务器（复杂度高）

### 向后兼容

如果有用户依赖当前的 base64 行为：
- 添加配置选项 `multimodal.force_base64 = false`（默认）
- 保留 base64 转换代码，但默认不使用

## 📚 相关文件

- `src/agent/loop_.rs:2105-2130` - 主要修改点
- `src/multimodal.rs` - 可能删除或大幅简化
- `src/channels/*/` - 各 channel 的 send() 方法
- `src/providers/traits.rs:387-395` - Vision 能力定义

## 🎉 总结

**核心思想**：
1. ✅ 工具输出的图片 → 保持本地路径
2. ✅ 用户上传的图片 → 按需转换 base64（仅 vision provider）
3. ✅ Channel 利用本地能力（剪贴板、文件系统）
4. ✅ 移除不必要的转换（性能 + 用户体验）

**收益**：
- 🚀 性能提升 100 倍（无 base64 编码）
- 💾 内存节省 100 倍（路径 vs base64 字符串）
- 💰 Token 节省 700 倍（10 vs 7000 tokens）
- 😊 用户体验改善（文件路径 vs base64 字符串）
