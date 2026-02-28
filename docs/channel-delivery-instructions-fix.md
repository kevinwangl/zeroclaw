# Channel Delivery Instructions 通用默认指令实现

## 🎯 问题总结

### 发现的问题
在分析钉钉图片发送问题时，发现了一个**系统性问题**：

- ✅ **仅 Telegram** 有 channel delivery instructions
- ❌ **其他 19 个 channel** 都返回 `None`
- 结果：LLM 在不同 channel 上的行为**严重不一致**

### 影响范围

| 受影响 Channel | 数量 | 问题表现 |
|---------------|------|---------|
| 已实现附件支持 | 6 个 | LLM 冗长解释工具结果，而非简洁使用媒体标记 |
| 未实现附件支持 | 10+ 个 | 未来实现附件时会遇到相同问题 |
| 特殊用途 | 3 个 | cli/dummy/ClawdTalk 不需要指令 |

**受影响的主要 channel**：
- Discord, Slack, Mattermost, Matrix (已有附件支持)
- DingTalk, Lark/Feishu (已有附件支持)
- Signal, IRC, iMessage, WhatsApp, QQ, Nostr, Email 等

## ✅ 实施的解决方案

### 方案选择：通用默认指令（方案 2）

**原因**：
1. **快速修复**：一次性解决所有 channel 的基本问题
2. **低维护成本**：统一的默认行为
3. **向后兼容**：不影响 Telegram 的现有行为
4. **可扩展**：未来可以为特定 channel 添加专门指令

### 代码变更

#### 修改文件：`src/channels/mod.rs`

```rust
fn channel_delivery_instructions(channel_name: &str) -> Option<&'static str> {
    match channel_name {
        "telegram" => Some("...Telegram 专门指令..."),
        
        // 特殊 channel 不需要指令
        "cli" | "dummy" | "ClawdTalk" => None,
        
        // 所有其他 channel 使用默认指令
        _ => Some(
            "When responding:\n\
             - Be concise and direct. Skip filler phrases like 'Great question!' or 'Certainly!'\n\
             - For media attachments use markers: [IMAGE:<path-or-url>], [DOCUMENT:<path-or-url>], [VIDEO:<path-or-url>], [AUDIO:<path-or-url>], or [VOICE:<path-or-url>]\n\
             - Keep normal text outside markers and never wrap markers in code fences\n\
             - Use tool results silently: answer the latest user message directly, and do not narrate delayed/internal tool execution bookkeeping",
        ),
    }
}
```

#### 新增文件：`src/channels/delivery_instructions_tests.rs`

添加了 17 个测试用例，覆盖：
- ✅ Telegram 保持专门指令
- ✅ Discord/Slack/Mattermost/Matrix/DingTalk/Lark 等获得默认指令
- ✅ cli/dummy/ClawdTalk 不获得指令
- ✅ 默认指令包含所有媒体标记
- ✅ 默认指令强调简洁性
- ✅ 默认指令指导工具结果使用

## 📊 默认指令内容分析

### 核心要点

1. **简洁性指导**
   ```
   "Be concise and direct. Skip filler phrases like 'Great question!' or 'Certainly!'"
   ```
   - 避免冗长的礼貌用语
   - 直接回答问题

2. **媒体标记使用**
   ```
   "For media attachments use markers: [IMAGE:<path-or-url>], [DOCUMENT:<path-or-url>], ..."
   ```
   - 明确告知 LLM 如何标记附件
   - 支持所有 5 种附件类型

3. **标记格式规范**
   ```
   "Keep normal text outside markers and never wrap markers in code fences"
   ```
   - 防止 LLM 将标记包裹在代码块中
   - 确保标记可以被正确解析

4. **工具结果处理**
   ```
   "Use tool results silently: answer the latest user message directly, 
    and do not narrate delayed/internal tool execution bookkeeping"
   ```
   - **核心修复点**：告诉 LLM 不要解释工具执行过程
   - 直接使用工具结果回答用户问题

## 🔄 行为变化对比

### 修复前（DingTalk 示例）

```
用户: 截图
LLM: [调用 screenshot 工具]
工具结果: [IMAGE:/tmp/screenshot.png]
LLM 回复: "截图已经成功生成了！图片已经包含在我的回复中（以 标记的形式）。
          从截图可以看到这是一个 macOS 系统的桌面环境，显示了多个应用窗口和界面。
          如果你在钉钉客户端中看不到图片，可能是因为：
          1. 钉钉机器人的消息格式限制
          2. 图片数据需要特定的格式才能在钉钉中显示
          截图功能本身是正常工作的，图片已经保存在工作目录中（大小约 3.9MB）。
          如果需要，我可以帮你用其他方式处理这个截图文件。
          📎 IMAGE: /tmp/screenshot.png
          Note: File upload requires DingTalk Open API integration"
```

**问题**：
- ❌ 冗长的解释（200+ 字）
- ❌ 解释工具执行细节
- ❌ 猜测用户可能遇到的问题
- ❌ 用户体验差

### 修复后（预期行为）

```
用户: 截图
LLM: [调用 screenshot 工具]
工具结果: [IMAGE:/tmp/screenshot.png]
LLM 回复: "截图已保存 [IMAGE:/tmp/screenshot.png]"
DingTalk: [解析标记] → [发送回退消息]
```

**改进**：
- ✅ 简洁回复（10 字以内）
- ✅ 不解释工具执行
- ✅ 直接传递标记
- ✅ 更好的用户体验

## 📈 测试结果

```bash
cargo test --lib channels::delivery_instructions_tests

running 17 tests
test ... clawdtalk_has_no_instructions ... ok
test ... cli_has_no_instructions ... ok
test ... default_instructions_contain_media_markers ... ok
test ... discord_has_default_instructions ... ok
test ... dummy_has_no_instructions ... ok
test ... default_instructions_guide_tool_result_usage ... ok
test ... feishu_has_default_instructions ... ok
test ... lark_has_default_instructions ... ok
test ... default_instructions_emphasize_conciseness ... ok
test ... dingtalk_has_default_instructions ... ok
test ... matrix_has_default_instructions ... ok
test ... qq_has_default_instructions ... ok
test ... mattermost_has_default_instructions ... ok
test ... signal_has_default_instructions ... ok
test ... slack_has_default_instructions ... ok
test ... telegram_has_specific_instructions ... ok
test ... whatsapp_has_default_instructions ... ok

test result: ok. 17 passed; 0 failed; 0 ignored
```

✅ **所有测试通过**

## 📝 文件变更总结

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `src/channels/mod.rs` | 修改 | 添加默认指令分支 |
| `src/channels/delivery_instructions_tests.rs` | 新增 | 17 个测试用例 |

**代码行数**：
- 新增：~150 行（测试）
- 修改：~10 行（逻辑）

## 🎯 影响评估

### 受益 Channel（19 个）

现在获得默认指令的 channel：
1. discord
2. slack
3. mattermost
4. matrix
5. dingtalk
6. lark / feishu
7. signal
8. irc
9. imessage
10. whatsapp / whatsapp_web
11. qq
12. nostr
13. email
14. linq
15. wati
16. nextcloud_talk
17. 以及未来新增的任何 channel

### 不受影响 Channel（4 个）

保持原有行为：
- telegram（保持专门指令）
- cli（不需要指令）
- dummy（测试用，不需要指令）
- ClawdTalk（语音通话，不需要指令）

## 🔒 向后兼容性

- ✅ **Telegram 行为不变**：保持现有专门指令
- ✅ **特殊 channel 不变**：cli/dummy/ClawdTalk 仍返回 None
- ✅ **无破坏性变更**：只是添加了之前缺失的指令
- ✅ **渐进增强**：改善了 LLM 行为，不影响现有功能

## 🚀 预期效果

### 立即改善

1. **DingTalk**：LLM 不再冗长解释，简洁使用媒体标记
2. **Discord/Slack/Mattermost/Matrix**：LLM 行为更一致
3. **Lark**：LLM 更好地利用附件上传功能
4. **所有 channel**：统一的简洁回复风格

### 长期收益

1. **一致性**：所有 channel 的 LLM 行为基本一致
2. **可维护性**：新增 channel 自动获得默认指令
3. **可扩展性**：未来可以为特定 channel 添加专门指令
4. **用户体验**：更简洁、更直接的回复

## 📚 后续优化建议

### 短期（可选）

为已实现附件支持的 channel 添加平台特定指令：
- Discord：提及 Discord 的 Markdown 支持
- Slack：提及 Slack 的格式规范
- Matrix：提及 Matrix 的加密房间支持

### 长期（可选）

为每个 channel 添加完整的平台特定指令（类似 Telegram）：
- 格式化指导（bold/italic/code）
- 平台特性说明
- 最佳实践建议

## 🎉 总结

通过添加通用默认指令，我们：

1. ✅ **修复了系统性问题**：19 个 channel 从无指令变为有指令
2. ✅ **改善了 LLM 行为**：简洁、一致、用户友好
3. ✅ **保持了向后兼容**：Telegram 和特殊 channel 不受影响
4. ✅ **提供了测试覆盖**：17 个测试确保正确性
5. ✅ **为未来铺平道路**：新 channel 自动获得默认指令

**这是一个低成本、高收益的快速修复方案，立即解决了所有 channel 的基本问题。**
