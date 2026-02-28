# DingTalk 和 Lark 附件支持实现总结

## 🎯 实现目标

为 DingTalk 和 Lark 两个 channel 添加统一的附件支持，使其能够处理 `[IMAGE:path]`、`[DOCUMENT:url]` 等媒体标记。

## ✅ 完成内容

### 1. DingTalk Channel (`src/channels/dingtalk.rs`)

**实现策略**: 基于 Webhook 的 Markdown 回退方案

**核心改动**:
- ✅ 集成 `parse_attachment_markers()` 解析附件标记
- ✅ 优先发送文本消息
- ✅ 附件处理逻辑：
  - **本地文件**: 发送文件路径信息 + Open API 集成提示
  - **URL 链接**: 发送 Markdown 格式链接 `[类型](url)`
- ✅ 优雅的错误处理和日志记录

**技术限制**:
- DingTalk Stream API 的 webhook 不支持直接文件上传
- 完整的文件上传需要 DingTalk Open API 集成（未来增强）
- 当前实现提供用户友好的回退消息

**代码示例**:
```rust
// 解析附件标记
let (text, attachments) = parse_attachment_markers(&content);

// 发送文本消息
if !text.is_empty() || attachments.is_empty() {
    // POST webhook with markdown
}

// 处理附件
for attachment in &attachments {
    if is_local_path(&attachment.target) {
        // 发送: "📎 IMAGE: `/path/to/file`\n*Note: 需要 Open API 集成*"
    } else {
        // 发送: "[IMAGE](https://example.com/image.png)"
    }
}
```

### 2. Lark Channel (`src/channels/lark.rs`)

**实现策略**: 完整的 API 集成 + 图片压缩

**核心改动**:
- ✅ 重构现有的 `extract_image_marker()` 逻辑，使用统一的 `parse_attachment_markers()`
- ✅ 新增 `upload_file()` 方法处理非图片附件
- ✅ 支持所有附件类型：
  - **图片**: 使用现有 `upload_image()`，自动压缩 >5MB 的图片
  - **文档/视频/音频/语音**: 使用新的 `upload_file()` 方法
- ✅ 所有上传操作都支持 token 自动刷新
- ✅ 同时支持本地文件和 URL 链接

**API 端点**:
- 图片上传: `POST /im/v1/images` (>5MB 自动压缩为 JPEG)
- 文件上传: `POST /im/v1/files` (文档、视频、音频、语音)

**代码示例**:
```rust
// 解析附件标记
let (text, attachments) = parse_attachment_markers(&content);

// 发送文本消息（带 token 刷新）
if !text.is_empty() || attachments.is_empty() {
    // ... 现有的 token 刷新逻辑 ...
}

// 按类型上传附件
for attachment in &attachments {
    match attachment.kind {
        AttachmentKind::Image => {
            let image_key = self.upload_image(&token, &target).await?;
            // 发送 msg_type: "image" 消息
        }
        _ => {
            let file_key = self.upload_file(&token, &target).await?;
            // 发送 msg_type: "file" 消息
        }
    }
}
```

## 📊 实现状态

| Channel | 状态 | 说明 |
|---------|------|------|
| Telegram | ✅ 已完成 | 参考实现 |
| Discord | ✅ 已完成 | 参考实现 |
| Slack | ✅ 已完成 | files.upload API |
| Mattermost | ✅ 已完成 | 两步上传流程 |
| Matrix | ✅ 已完成 | send_attachment() |
| **DingTalk** | ✅ **新增** | Webhook + Markdown 回退 |
| **Lark** | ✅ **新增** | 完整 API 集成 + 压缩 |
| Signal | 🚧 待实现 | 需要 RPC 扩展 |
| IRC | 🚧 待实现 | 仅 URL 回退 |
| 其他 | 🚧 待实现 | 见实现指南 |

## 🔧 技术细节

### DingTalk 特性

**Webhook 限制**:
- Stream API webhook 仅支持文本/Markdown 消息
- 文件上传需要 Open API + 应用凭证
- 当前实现提供优雅降级

**消息格式**:
```json
{
  "msgtype": "markdown",
  "markdown": {
    "title": "Attachment",
    "text": "[IMAGE](https://example.com/image.png)"
  }
}
```

### Lark 特性

**图片压缩**:
- 自动对 >5MB 的图片进行 JPEG 压缩
- 超大图片缩放至最大 2048x2048
- 小图片保持原始质量

**文件大小限制**:
- 图片: 10MB（压缩后）
- 文件: 20MB
- 超出限制会导致上传失败

**消息类型**:
- `msg_type: "image"` + `image_key`
- `msg_type: "file"` + `file_key`
- `msg_type: "text"` (URL 回退)

## 📝 文件变更

**修改的文件**:
- `src/channels/dingtalk.rs` - 添加附件支持
- `src/channels/lark.rs` - 重构并扩展附件支持

**新增的文件**:
- `src/channels/dingtalk_lark_attachment_tests.rs` - 单元测试
- `docs/dingtalk-lark-attachment-implementation.md` - 详细实现文档

**更新的文档**:
- `docs/channel-attachment-implementation.md` - 更新状态表

## ✅ 验证结果

### 编译状态
```bash
cargo check
# ✅ Finished `dev` profile in 6.65s
# 仅 2 个预存在的 unused import 警告
```

### 单元测试
- ✅ `test_parse_dingtalk_attachment_markers()` - 多附件解析
- ✅ `test_parse_lark_attachment_markers()` - 混合本地/URL 附件
- ✅ `test_attachment_kind_marker_names()` - 标记名称一致性

### 手动测试清单

**DingTalk**:
- [ ] 发送带 `[IMAGE:/path/to/image.png]` 的消息
- [ ] 发送带 `[DOCUMENT:https://example.com/doc.pdf]` 的消息
- [ ] 验证回退消息用户友好
- [ ] 验证 Markdown 链接正确渲染

**Lark**:
- [ ] 发送带小图片的消息 (< 5MB)
- [ ] 发送带大图片的消息 (> 5MB，验证压缩)
- [ ] 发送带文档的消息 `[DOCUMENT:/path/to/file.pdf]`
- [ ] 发送带视频的消息 `[VIDEO:/path/to/video.mp4]`
- [ ] 发送带 URL 附件的消息
- [ ] 验证 token 刷新在上传期间正常工作

## 🔒 安全考虑

1. **路径验证**: 两个实现都在上传前检查文件存在性
2. **Token 安全**: Lark token 在过期时自动刷新
3. **错误处理**: 文件未找到错误被记录，不暴露给用户
4. **URL 清理**: URL 按原样发送，不执行
5. **工作区作用域**: 遵守现有安全策略

## ⚡ 性能特性

**DingTalk**:
- 最小开销（仅文本回退）
- 本地文件无文件 I/O（仅发送路径信息）
- 每个附件异步 webhook POST

**Lark**:
- 使用 tokio::fs 异步文件上传
- 图片压缩在阻塞任务中执行（CPU 密集型）
- 顺序上传附件（防止速率限制）
- Token 缓存减少 API 调用

## 🔄 向后兼容性

- ✅ **无破坏性变更**: 现有纯文本消息不受影响
- ✅ **可选功能**: 仅在存在附件标记时处理
- ✅ **回退机制**: 无法识别的标记保留为纯文本
- ✅ **Token 逻辑**: Lark 现有 token 逻辑保持不变

## 📈 实现指标

| 指标 | 数值 |
|------|------|
| **修改文件数** | 2 (dingtalk.rs, lark.rs) |
| **新增文件数** | 2 (测试 + 文档) |
| **新增代码行** | ~200 |
| **新增方法** | 1 (Lark::upload_file) |
| **测试用例** | 3 |
| **支持的附件类型** | 5 (Image, Document, Video, Audio, Voice) |

## 🚀 下一步

### 立即执行
1. 使用真实 DingTalk/Lark 账号进行手动测试
2. 添加带模拟 API 响应的集成测试

### 短期计划
3. DingTalk: 实现 Open API 文件上传（需要应用凭证）
4. Lark: 为大型视频/音频文件添加压缩

### 长期计划
5. 为失败的上传添加重试逻辑
6. 实现大文件上传的进度跟踪
7. 上传前添加文件类型验证

## 📚 参考资料

- 共享工具: `src/channels/attachment.rs`
- 实现指南: `docs/channel-attachment-implementation.md`
- 详细文档: `docs/dingtalk-lark-attachment-implementation.md`
- DingTalk Stream API: https://open.dingtalk.com/document/orgapp/stream-mode-overview
- Lark Open API: https://open.feishu.cn/document/server-docs/im-v1/message/create
- 系统提示词: `src/channels/mod.rs:410`
