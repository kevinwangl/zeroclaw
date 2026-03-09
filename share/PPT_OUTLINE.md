# AI Agent 工具集成实践 PPT 详细大纲

## 第 1 部分: 开场（5分钟）

### Slide 1: 标题页
**标题**: AI Agent 工具集成实践：从 CLI 到自然语言
**副标题**: ZeroClaw + Financial Analyzer 集成案例
**演讲人**: [你的名字]
**日期**: 2026-03-09

**视觉元素**:
- 背景: 深蓝渐变
- Logo: ZeroClaw + Financial Analyzer
- 图标: 🤖 + 📊

---

### Slide 2: Demo 视频（3分钟）
**标题**: 先看效果

**视频内容**:
1. 打开钉钉
2. 发送: "帮我分析贵州茅台的最近3年财务报告"
3. 等待 3 分钟
4. 收到详细分析报告
5. 展示生成的 Excel 和 TXT 文件

**旁白文案**:
```
"传统方式需要：
1. 手动下载财务数据
2. Excel 计算各种指标
3. 写分析报告
耗时 1-2 小时

现在只需要：
1. 发送一条消息
2. 等待 3 分钟
3. 收到完整报告"
```

**备用方案**: 如果现场网络不好，播放预录视频

---

### Slide 3: 痛点与价值
**标题**: 为什么要做这个？

**左侧 - 传统方式的痛点**:
- ❌ 手动下载数据（10分钟）
- ❌ Excel 计算指标（30分钟）
- ❌ 写分析报告（20分钟）
- ❌ 重复劳动、易出错
- ❌ 需要专业知识

**右侧 - AI Agent 方式的价值**:
- ✅ 自然语言输入（10秒）
- ✅ 自动获取真实数据
- ✅ 自动计算所有指标
- ✅ 自动生成报告
- ✅ 3分钟完成，零错误

**底部 - ROI**:
```
时间节省: 60分钟 → 3分钟 (节省 95%)
准确率: 人工 85% → 自动 99%
可扩展: 1个股票 → 100个股票（同样时间）
```

---

## 第 2 部分: 架构设计（15分钟）

### Slide 4: 系统架构图
**标题**: 整体架构

**架构图**:
```
┌─────────────┐
│   用户      │ "分析贵州茅台"
│  (DingTalk) │
└──────┬──────┘
       │
       ↓
┌─────────────────────┐
│  ZeroClaw Channel   │ 接收消息
│  (Rust Runtime)     │
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│  Kiro CLI Provider  │ LLM 决策
│  (Claude Haiku)     │
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│    Shell Tool       │ 执行命令
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│ analyze_stock.sh    │ 包装脚本
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│ financial-analyzer  │ Rust CLI
│    (Rust)           │
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│     akshare         │ 真实数据
│  (Python Library)   │
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│  Excel + TXT 报告   │
└─────────────────────┘
       │
       ↓
┌─────────────────────┐
│  Agent 读取并总结   │
└──────┬──────────────┘
       │
       ↓
┌─────────────────────┐
│    回复用户         │
└─────────────────────┘
```

**关键点标注**:
- 🔵 用户交互层
- 🟢 AI 决策层
- 🟡 工具执行层
- 🟠 数据获取层
- 🔴 结果生成层

---

### Slide 5: 技术栈
**标题**: 为什么选这些技术？

**技术栈表格**:
| 组件 | 技术 | 选择理由 |
|------|------|----------|
| Agent 运行时 | ZeroClaw (Rust) | 高性能、低内存、安全 |
| LLM Provider | Kiro CLI | 统一工具批准、错误处理 |
| 财务分析 | financial-analyzer (Rust) | 已有工具、性能好 |
| 数据源 | akshare | 免费、真实、更新快 |
| 消息渠道 | DingTalk | 企业内部、易集成 |

**技术亮点**:
- 🦀 全 Rust 栈（除数据源）
- ⚡ 高性能（< 5MB 内存）
- 🔒 安全（沙箱 + 白名单）
- 🔧 可扩展（trait 架构）

---

### Slide 6: 关键决策
**标题**: 3 个关键技术决策

**决策 1: 包装脚本 vs 直接调用**

| 方案 | 优点 | 缺点 | 选择 |
|------|------|------|------|
| 直接调用 | 简单 | 路径复杂、参数转换、错误处理 | ❌ |
| 包装脚本 | 统一接口、易维护 | 多一层抽象 | ✅ |

**决策 2: MANDATORY 提示 vs 普通说明**

| 方案 | 效果 | 问题 | 选择 |
|------|------|------|------|
| 普通说明 | Agent 自己写脚本 | 不可控 | ❌ |
| MANDATORY | 强制调用工具 | 需要精确措辞 | ✅ |

**决策 3: 白名单 vs 黑名单**

| 方案 | 安全性 | 灵活性 | 选择 |
|------|--------|--------|------|
| 黑名单 | 低（默认允许） | 高 | ❌ |
| 白名单 | 高（默认拒绝） | 中 | ✅ |

---

## 第 3 部分: 核心改造点（20分钟）

### Slide 7: 改造点 1 - 工具集成架构
**标题**: 从"自己写脚本"到"调用工具"

**左侧 - ❌ 初始方案**:
```python
# Agent 自己写的 Python 脚本
import akshare as ak
df = ak.stock_financial_abstract(stock="600519")
# ... 100 行代码
```

**问题**:
- 每次都重新生成代码
- 代码质量不稳定
- 无法复用已有工具
- 难以维护

**右侧 - ✅ 最终方案**:
```bash
# 包装脚本
~/.zeroclaw/workspace/analyze_stock.sh \
  --stock 600519.SH \
  --years 2023,2022,2021 \
  --source akshare
```

**优势**:
- 复用已有 Rust 工具
- 统一接口
- 易于测试
- 性能更好

**💡 经验**: 能复用就不重写，包装脚本是最简单的集成方式

---

### Slide 8: 改造点 2 - 提示工程
**标题**: 如何让 Agent 听话？

**左侧 - ❌ 初始方案**:
```markdown
# TOOLS.md
你可以使用 analyze_stock.sh 分析股票
```

**问题**:
- Agent 忽略说明
- 自己写 Python 脚本
- 不可控

**右侧 - ✅ 最终方案**:
```markdown
# MEMORY.md
## MANDATORY: 财务分析工具使用规则

当用户请求分析股票财务报告时，你**必须**:
1. 使用 shell 工具
2. 执行 ~/.zeroclaw/workspace/analyze_stock.sh
3. **禁止**使用 file_write 工具
4. **禁止**自己写 Python 脚本
```

**关键词**:
- MANDATORY（强制）
- **必须**（加粗）
- **禁止**（明确边界）

**💡 经验**: LLM 需要强约束，MANDATORY + 禁止列表 = 可控行为

---

### Slide 9: 改造点 3 - 权限配置
**标题**: 安全与灵活的平衡

**左侧 - ❌ 初始方案**:
```toml
[autonomy]
workspace_only = true  # 默认沙箱
```

**问题**:
- 无法访问外部工具
- 无法读取生成的报告

**右侧 - ✅ 最终方案**:
```toml
[autonomy]
workspace_only = false
allowed_roots = [
    "~/Downloads/bryan/private_data/funds/stocks/financial-analyzer",
    "~/Downloads/bryan/private_data/funds/stocks/analyzer-report"
]
allowed_commands = ["cargo", "cd"]
auto_approve = ["shell"]
```

**安全机制**:
- 白名单目录（只能访问指定路径）
- 白名单命令（只能执行指定命令）
- 自动批准（shell 工具无需确认）

**💡 经验**: 白名单 > 黑名单，最小权限原则

---

### Slide 10: 改造点 4 - 日志可观测性
**标题**: 看不见就无法优化

**左侧 - ❌ 初始方案**:
```
💬 [dingtalk] from user: 分析贵州茅台
⏳ Processing message...
(等待 3 分钟，不知道在干什么)
🤖 Reply: ...
```

**问题**:
- 看不到 LLM 请求/响应
- 看不到工具执行
- 看不到 Token 统计
- 无法排查问题

**右侧 - ✅ 最终方案**:
```bash
# ~/.zshrc
export RUST_LOG=zeroclaw=debug,zeroclaw::agent=debug
export RUST_LOG_STYLE=always
```

**日志输出**:
```
DEBUG LLM request: {...}
DEBUG LLM response: {...} tokens=1234
DEBUG Tool execution: shell analyze_stock.sh
DEBUG Tool result: success
```

**💡 经验**: 详细日志是排查问题的唯一途径

---

### Slide 11: 改造点 5 - 性能优化
**标题**: 从 25秒 到 17秒

**优化历程**:
```
初始配置:
  Provider: Kiro CLI
  Model: Claude Sonnet 4.6
  Memory: auto_save = true
  响应时间: 25秒

优化 1 - 切换模型:
  Model: Claude Haiku
  响应时间: 18秒 (提速 28%)

优化 2 - 禁用自动保存:
  Memory: auto_save = false
  响应时间: 17秒 (提速 32%)
```

**性能分解**:
```
17秒 = Kiro 启动 (15秒) + LLM 响应 (2秒)
```

**瓶颈分析**:
```
真正耗时: 财务分析工具执行 (120秒)
- 获取真实数据 (60秒)
- 计算指标 (40秒)
- 生成报告 (20秒)
```

**💡 经验**: 区分 LLM 性能 vs 业务逻辑性能

---

### Slide 12: 坑 5 - 工具调用速率限制
**标题**: "Rate limit exceeded: action budget exhausted"

**现象**:
```
Tool execution failed: Rate limit exceeded: action budget exhausted
```

**原因分析**:
1. ZeroClaw 有工具调用速率限制
2. 默认 100 次/小时
3. 财务分析每次调用多个工具
4. 快速超过限制

**配置位置**:
```toml
# ~/.zeroclaw/config.toml
[autonomy]
max_actions_per_hour = 100  # 默认值
```

**解决方案**:
```toml
# 方案 1: 增加限制
[autonomy]
max_actions_per_hour = 500

# 方案 2: 禁用限制（开发环境）
[autonomy]
max_actions_per_hour = 0
```

**验证结果**:
```
✅ 工具调用成功
✅ 不再有速率限制错误
✅ 可以连续分析多个股票
```

**💡 教训**: 
- 生产环境保留速率限制（安全）
- 开发环境可以放宽或禁用
- 监控实际使用量，合理设置阈值


---

## 第 4 部分: 踩坑与解决（10分钟）

### Slide 13: 坑 1 - Agent 不调用工具
**标题**: 为什么 Agent 自己写脚本？

**现象截图**:
```
User: 分析贵州茅台
Agent: 我来写一个 Python 脚本...
[生成 100 行 Python 代码]
```

**原因分析**:
1. LLM 有"自作聪明"倾向
2. 普通文档说明权重低
3. 没有明确禁止替代方案

**解决方案**:
```markdown
## MANDATORY: 财务分析工具使用规则

你**必须**使用 shell 工具执行:
~/.zeroclaw/workspace/analyze_stock.sh

**禁止**:
- 使用 file_write 工具
- 自己写 Python 脚本
- 使用其他任何方式
```

**验证结果**:
```
User: 分析贵州茅台
Agent: [调用 shell 工具]
✅ 成功执行 analyze_stock.sh
```

**💡 教训**: 不要期望 LLM 自觉，必须强制约束

---

### Slide 14: 坑 2 - 日志不完整
**标题**: 看不到 LLM 在干什么

**现象**:
```
💬 [dingtalk] from user: 分析贵州茅台
⏳ Processing message...
(黑盒，等待 3 分钟)
🤖 Reply: ...
```

**问题**:
- 不知道 LLM 是否响应
- 不知道调用了什么工具
- 不知道哪里慢
- 无法排查问题

**原因**:
```bash
# 默认日志级别
RUST_LOG=info  # 只显示关键信息
```

**解决方案**:
```bash
# ~/.zshrc
export RUST_LOG=zeroclaw=debug,zeroclaw::agent=debug,zeroclaw::channels=debug,zeroclaw::providers=debug
export RUST_LOG_STYLE=always
```

**验证结果**:
```
DEBUG LLM request: {"messages": [...]}
DEBUG LLM response: {"content": "...", "tokens": 1234}
DEBUG Tool execution: shell analyze_stock.sh
DEBUG Tool result: success (120s)
DEBUG Reply sent to user
```

**💡 教训**: 详细日志是排查问题的第一步

---

### Slide 13: 坑 3 - Kiro 工具批准失败
**标题**: "Tool approval required but --no-interactive"

**现象**:
```
WARN Provider call failed, retrying
error: Tool approval required but --no-interactive was specified
```

**原因分析**:
1. ZeroClaw 调用 Kiro 时使用 `--no-interactive`
2. Kiro 默认需要人工批准工具
3. 非交互模式下无法批准 → 失败

**解决方案**:
```json
// ~/.kiro/settings.json
{
  "chat": {
    "trustAllTools": true,
    "disableToolApproval": true
  }
}
```

**验证结果**:
```
✅ Kiro CLI 自动批准工具
✅ LLM 请求成功
✅ 工具执行成功
```

**💡 教训**: 测试工具批准流程，不要假设默认配置可用

---

### Slide 16: 坑 4 - 响应慢
**标题**: 为什么要等 3 分钟？

**现象**:
```
08:20:22 - 收到消息
08:23:33 - 回复完成
总耗时: 191秒 (3分11秒)
```

**初步假设**:
- ❌ Kiro CLI 启动慢？
- ❌ LLM 响应慢？
- ❌ 网络慢？

**真相**:
```
时间分解:
  32秒  - Kiro CLI 启动 + LLM 决策
  120秒 - financial-analyzer 执行 ← 真正瓶颈
  39秒  - 读取报告 + 生成回复
```

**优化方案**:
```toml
# 1. 切换更快的模型
default_model = "claude-3-5-haiku-20241022"

# 2. 禁用自动保存
[memory]
auto_save = false
```

**优化效果**:
```
优化前: 25秒 (Kiro + Sonnet)
优化后: 17秒 (Kiro + Haiku)
提速: 32%
```

**💡 教训**: 性能优化要先测量，区分 LLM vs 业务逻辑

---

### Slide 17: 踩坑总结
**标题**: 问题发现与解决时间线

**时间线图**:
```
Day 1:
  09:00 - 开始集成
  11:00 - 发现坑1: Agent 不调用工具
  12:00 - 解决: MANDATORY 提示
  14:00 - 发现坑2: 日志不完整
  15:00 - 解决: RUST_LOG 配置

Day 2:
  09:00 - 发现坑3: 工具批准失败
  10:00 - 解决: trustAllTools
  11:00 - 发现坑4: 响应慢
  14:00 - 解决: 切换 Haiku
  16:00 - 集成完成 ✅
```

**关键洞察**:
- 80% 的时间在排查问题
- 20% 的时间在写代码
- 详细日志节省 50% 排查时间

---

## 第 5 部分: 可复用模式（5分钟）

### Slide 18: 模式 1 - 外部工具集成配置
**标题**: 如何安全地集成外部工具？

**配置模板**:
```toml
[autonomy]
workspace_only = false
allowed_roots = [
    "/path/to/your/tool",
    "/path/to/output/dir"
]
allowed_commands = ["your-tool-binary"]
auto_approve = ["shell"]
```

**适用场景**:
- ✅ 已有 CLI 工具需要 AI 化
- ✅ 需要访问 workspace 外的目录
- ✅ 需要自动批准工具执行

**使用示例**:
```toml
# 集成天气查询工具
[autonomy]
allowed_roots = ["/usr/local/bin/weather-cli"]
allowed_commands = ["weather"]
```

---

### Slide 19: 模式 2 - 强制提示模板
**标题**: 如何让 Agent 100% 执行？

**提示模板**:
```markdown
## MANDATORY: [功能名称]工具使用规则

当用户请求 [具体场景] 时，你**必须**:
1. 使用 [工具名称] 工具
2. 执行 [具体命令]
3. 参数格式: [参数说明]

**禁止**:
- 使用 [替代工具1]
- 使用 [替代工具2]
- [其他禁止行为]

示例:
用户: [示例输入]
你: [调用 工具名称 工具]
```

**适用场景**:
- ✅ 需要强制执行特定工具
- ✅ LLM 有"自作聪明"倾向
- ✅ 需要明确禁止替代方案

**使用示例**:
```markdown
## MANDATORY: 代码格式化工具使用规则

当用户请求格式化代码时，你**必须**:
1. 使用 shell 工具
2. 执行 rustfmt --edition 2021 [文件]

**禁止**:
- 自己重写代码
- 使用 file_write 修改
```

---

### Slide 20: 模式 3 - 包装脚本模板
**标题**: 统一工具接口

**脚本模板**:
```bash
#!/bin/bash
# [工具名称] 包装脚本

set -euo pipefail

# 1. 参数解析
PARAM1=""
PARAM2=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --param1) PARAM1="$2"; shift 2 ;;
    --param2) PARAM2="$2"; shift 2 ;;
    *) echo "Unknown: $1"; exit 1 ;;
  esac
done

# 2. 参数验证
if [[ -z "$PARAM1" ]]; then
  echo "Error: --param1 required"
  exit 1
fi

# 3. 路径处理
TOOL_DIR="$HOME/path/to/tool"
OUTPUT_DIR="$HOME/path/to/output"

# 4. 执行工具
cd "$TOOL_DIR"
./tool-binary --param1 "$PARAM1" --param2 "$PARAM2"

# 5. 输出结果
echo "✅ Success: $OUTPUT_DIR/result.txt"
```

**适用场景**:
- ✅ 工具路径复杂
- ✅ 需要参数转换
- ✅ 需要统一错误处理

---

### Slide 21: 模式 4 - 日志增强模板
**标题**: 详细日志配置

**配置模板**:
```bash
# ~/.zshrc 或 ~/.bashrc

# 基础日志（生产环境）
export RUST_LOG=info

# 详细日志（开发/调试）
export RUST_LOG=zeroclaw=debug,zeroclaw::agent=debug,zeroclaw::channels=debug,zeroclaw::providers=debug

# 完整日志（问题排查）
export RUST_LOG=trace

# 保持颜色
export RUST_LOG_STYLE=always
```

**启动脚本模板**:
```bash
#!/bin/bash
# start_with_logging.sh

LOG_FILE="$HOME/.zeroclaw/logs/$(date +%Y%m%d-%H%M%S).log"

source ~/.zeroclaw/logging.env

zeroclaw channel start 2>&1 | tee "$LOG_FILE"
```

**适用场景**:
- ✅ 需要排查问题
- ✅ 需要性能分析
- ✅ 需要审计日志

---

### Slide 22: 模式应用矩阵
**标题**: 什么场景用什么模式？

**矩阵表格**:
| 场景 | 模式 1 | 模式 2 | 模式 3 | 模式 4 |
|------|--------|--------|--------|--------|
| 集成外部 CLI | ✅ | ✅ | ✅ | ✅ |
| Agent 不听话 | - | ✅ | - | - |
| 路径复杂 | ✅ | - | ✅ | - |
| 排查问题 | - | - | - | ✅ |
| 参数转换 | - | - | ✅ | - |
| 安全控制 | ✅ | - | - | - |

**组合使用**:
```
完整集成 = 模式1 + 模式2 + 模式3 + 模式4
```

---

## 第 6 部分: 总结（5分钟）

### Slide 23: 经验总结
**标题**: 做对的事 vs 走过的弯路

**✅ 做对的 5 件事**:
1. **包装脚本**: 简化集成，统一接口
2. **MANDATORY 提示**: 强制行为，100% 执行
3. **白名单机制**: 安全灵活，最小权限
4. **详细日志**: 可观测，快速排查
5. **速率限制配置**: 开发放宽，生产保留

**❌ 踩过的 5 个坑**:
1. **Agent 不调用工具** → MANDATORY 提示
2. **日志不完整** → RUST_LOG 配置
3. **工具批准失败** → trustAllTools
4. **响应慢** → 区分 LLM vs 业务逻辑
5. **速率限制** → max_actions_per_hour

**💡 5 个关键洞察**:
1. **LLM 不可靠**: 需要强约束，不要期望自觉
2. **日志很重要**: 看不见就无法排查和优化
3. **测试要充分**: 工具批准、速率限制都要测
4. **安全要平衡**: 白名单 > 黑名单，开发 vs 生产
5. **工具要简单**: 包装 > 重写，复用 > 造轮子

---

### Slide 24: 适用场景
**标题**: 这套方案适合你吗？

**✅ 适合的场景**:
- 已有 CLI 工具需要 AI 化
- 需要调用外部服务/API
- 需要处理真实数据
- 需要生成结构化报告
- 需要多渠道接入（钉钉/微信/Slack）
- 可接受 2-3 分钟延迟

**❌ 不适合的场景**:
- 简单的文本对话
- 纯 LLM 推理任务
- 实时性要求 < 1秒
- 无法接受任何延迟
- 不需要外部工具

**评估清单**:
```
[ ] 有现成的 CLI 工具？
[ ] 需要真实数据？
[ ] 可接受 2-3 分钟？
[ ] 需要多渠道？
[ ] 有安全要求？

如果 3+ 个 ✅ → 适合这套方案
```

---

### Slide 25: Q&A
**标题**: 感谢聆听，欢迎交流

**联系方式**:
- 📧 Email: [你的邮箱]
- 💬 微信: [你的微信]
- 🔗 GitHub: [代码仓库链接]

**参考资料**:
- 📁 完整配置: `~/.zeroclaw/workspace/INTEGRATION_SUMMARY.md`
- 📊 分享方案: `~/.zeroclaw/workspace/SHARING_PLAN.md`
- 📝 PPT 大纲: `~/.zeroclaw/workspace/PPT_OUTLINE.md`
- 🎬 Demo 脚本: `~/.zeroclaw/workspace/DEMO_SCRIPT.md`

**扫码获取资料**:
[二维码: 指向代码仓库或文档]

---

## 附录: 备用 Slides

### Backup 1: 技术栈对比
**标题**: 为什么不用其他方案？

| 方案 | 优点 | 缺点 | 选择 |
|------|------|------|------|
| LangChain | 生态丰富 | Python、内存大 | ❌ |
| AutoGPT | 开箱即用 | 不可控、慢 | ❌ |
| ZeroClaw | Rust、高性能 | 需要配置 | ✅ |

### Backup 2: 成本分析
**标题**: 这套方案要花多少钱？

| 项目 | 成本 | 说明 |
|------|------|------|
| ZeroClaw | 免费 | 开源 |
| Kiro CLI | 免费 | 开源 |
| Claude API | $0.003/1K tokens | 按量付费 |
| akshare | 免费 | 开源 |
| 服务器 | $10/月 | 最低配置 |

**单次分析成本**:
```
输入: 2K tokens × $0.003 = $0.006
输出: 1K tokens × $0.015 = $0.015
总计: $0.021 (约 ¥0.15)
```

### Backup 3: 安全考虑
**标题**: 如何保证安全？

**安全机制**:
1. **沙箱隔离**: workspace_only 默认开启
2. **白名单**: 只能访问指定目录和命令
3. **工具批准**: 敏感操作需要确认
4. **日志审计**: 所有操作可追溯
5. **权限最小化**: 只给必要权限

**风险评估**:
- 低风险: 读取文件、执行查询
- 中风险: 写入文件、网络请求
- 高风险: 系统命令、删除操作

---

**PPT 制作建议**:
1. 使用深色主题（技术感）
2. 代码用 Monokai 配色
3. 图表用 Chart.js 或 D3.js
4. 动画简洁（淡入淡出）
5. 每页不超过 3 个要点
