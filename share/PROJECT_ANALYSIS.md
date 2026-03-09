# ZeroClaw + Financial Analyzer 集成实践分析

## 项目背景

**目标**: 将 Rust CLI 财务分析工具集成到 AI Agent 系统，实现自然语言驱动的股票财务分析

**技术栈**:
- ZeroClaw: Rust 编写的 AI Agent 运行时
- Financial Analyzer: Rust CLI 财务分析工具
- Kiro CLI: LLM Provider（Claude）
- DingTalk: 消息渠道

## 核心改造点

### 1. 工具集成架构
- ❌ 初始方案: Agent 自己写 Python 脚本
- ✅ 最终方案: 预置 Bash 包装脚本 + shell 工具调用

### 2. 提示工程
- ❌ 初始: 普通文档说明
- ✅ 最终: MANDATORY 强制指令 + 明确禁止替代方案

### 3. 权限配置
- ❌ 初始: workspace_only = true（默认沙箱）
- ✅ 最终: allowed_roots 白名单机制

### 4. 日志可观测性
- ❌ 初始: 默认 info 级别，看不到细节
- ✅ 最终: debug 级别 + 环境变量持久化

### 5. 性能优化
- ❌ 初始: Kiro + Sonnet (25秒响应)
- ✅ 最终: Kiro + Haiku (17秒响应)

## 关键技术决策

### 决策 1: 为什么用包装脚本而不是直接调用？
- 路径管理复杂（相对路径 vs 绝对路径）
- 参数转换（股票代码格式）
- 输出文件命名规范
- 错误处理统一

### 决策 2: 为什么用 MANDATORY 提示？
- LLM 有"自作聪明"倾向
- 普通说明容易被忽略
- 需要强制约束行为

### 决策 3: 为什么不用 MCP？
- 工具已存在，无需重写
- Shell 调用更简单直接
- 减少依赖复杂度

### 决策 4: 为什么选 Kiro 而不是原生 Provider？
- 统一工具批准机制
- 更好的错误处理
- 与现有工作流一致

## 遇到的坑

### 坑 1: Agent 不调用工具
**现象**: Agent 自己写 Python 脚本
**原因**: 提示不够强
**解决**: MANDATORY + 禁止 file_write

### 坑 2: 日志看不到 LLM 交互
**现象**: 只看到 "Processing..."，不知道在干什么
**原因**: RUST_LOG 未配置
**解决**: 环境变量 + 启动脚本

### 坑 3: Kiro 工具批准失败
**现象**: "Tool approval required but --no-interactive"
**原因**: Kiro 默认需要人工批准
**解决**: trustAllTools: true

### 坑 4: 响应慢
**现象**: 3分钟才回复
**原因**: Kiro 启动开销 + 真实数据获取
**解决**: 切换 Haiku + 禁用 auto_save

## 架构图

```
用户 (DingTalk)
    ↓
ZeroClaw Channel
    ↓
Kiro CLI Provider
    ↓
Claude API (Haiku)
    ↓
Shell Tool
    ↓
analyze_stock.sh
    ↓
financial-analyzer (Rust)
    ↓
akshare (真实数据)
    ↓
Excel + TXT 报告
    ↓
Agent 读取并总结
    ↓
回复用户
```

## 性能优化历程

| 阶段 | 配置 | 响应时间 | 优化点 |
|------|------|----------|--------|
| 初始 | Kiro + Sonnet | 25秒 | - |
| 优化1 | Kiro + Haiku | 17秒 | 切换模型 |
| 优化2 | auto_save=false | 17秒 | 减少上下文 |
| 瓶颈 | 财务分析 | 120秒 | 业务逻辑 |

## 可复用的模式

### 模式 1: 外部工具集成
```toml
[autonomy]
workspace_only = false
allowed_roots = ["/path/to/tool"]
allowed_commands = ["tool-name"]
auto_approve = ["shell"]
```

### 模式 2: 强制提示
```markdown
## MANDATORY: 工具使用规则

当用户请求 X 时，你**必须**:
1. 使用 shell 工具
2. 执行 /path/to/script.sh
3. 禁止使用 file_write
4. 禁止写 Python 脚本
```

### 模式 3: 包装脚本
```bash
#!/bin/bash
# 统一参数处理
# 统一路径管理
# 统一错误处理
# 统一输出格式
```

### 模式 4: 日志增强
```bash
export RUST_LOG=zeroclaw=debug
export RUST_LOG_STYLE=always
```

## 经验总结

### ✅ 做对的事
1. 用包装脚本简化集成
2. 用 MANDATORY 强制行为
3. 用白名单而非黑名单
4. 用详细日志排查问题
5. 用真实数据验证

### ❌ 走过的弯路
1. 期望 Agent 自己写脚本
2. 用普通文档说明
3. 没有配置日志
4. 没有测试工具批准
5. 没有性能基准

### 💡 关键洞察
1. **LLM 不可靠**: 需要强约束
2. **日志很重要**: 看不见就无法优化
3. **性能有瓶颈**: 区分 LLM vs 业务逻辑
4. **安全要平衡**: 白名单 > 完全开放
5. **工具要简单**: 包装 > 重写

## 下一步改进

### 短期
- [ ] 缓存已分析的股票
- [ ] 异步处理（先回复"分析中"）
- [ ] 错误重试机制

### 中期
- [ ] 切换到原生 Provider（提速 8x）
- [ ] 实现流式响应
- [ ] 添加进度反馈

### 长期
- [ ] 预计算热门股票
- [ ] 分布式分析
- [ ] 实时数据更新

## 适用场景

这套方案适合:
- ✅ 已有 CLI 工具需要 AI 化
- ✅ 需要调用外部服务
- ✅ 需要处理真实数据
- ✅ 需要生成结构化报告
- ✅ 需要多渠道接入

不适合:
- ❌ 简单的文本对话
- ❌ 纯 LLM 推理任务
- ❌ 实时性要求 < 1秒
- ❌ 无法接受 2-3 分钟延迟

## 参考资料

- ZeroClaw 文档: README.md
- Financial Analyzer: 工具文档
- 配置文件: ~/.zeroclaw/config.toml
- 集成总结: INTEGRATION_SUMMARY.md
