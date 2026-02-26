# 快速配置 Channel（解决 "No real-time channels configured"）

## 问题

```
INFO zeroclaw::daemon: No real-time channels configured; channel supervisor disabled
```

这表示 ZeroClaw daemon 没有配置任何实时通信渠道。

## 解决方案

### 方案 1：配置 Telegram（推荐，最简单）

#### 1. 创建 Telegram Bot

1. 在 Telegram 中找到 [@BotFather](https://t.me/botfather)
2. 发送 `/newbot`
3. 按提示设置 bot 名称
4. 获取 bot token（格式：`123456:ABC-DEF...`）

#### 2. 获取你的 Telegram 用户名

- 在 Telegram 设置中查看你的 `@username`（不带 `@`）

#### 3. 配置 ZeroClaw

```bash
cat >> ~/.zeroclaw/config.toml <<'EOF'

[channels_config.telegram]
bot_token = "YOUR_BOT_TOKEN_HERE"
allowed_users = ["your_username"]  # 不带 @
mention_only = false
EOF
```

#### 4. 启动 daemon

```bash
zeroclaw daemon
```

#### 5. 测试

在 Telegram 中：
1. 搜索你的 bot
2. 点击 "Start"
3. 发送消息："Hello"
4. Bot 会通过 Kiro Provider 回复

---

### 方案 2：仅使用 CLI（无需配置）

如果你只想在命令行中使用，不需要配置 channels：

```bash
# 直接使用 agent 命令
zeroclaw agent --provider kiro -m "Hello"

# 交互模式
zeroclaw agent --provider kiro
```

**不需要运行 `zeroclaw daemon`**

---

### 方案 3：配置 Discord

#### 1. 创建 Discord Bot

1. 访问 [Discord Developer Portal](https://discord.com/developers/applications)
2. 创建新应用
3. 在 "Bot" 标签页创建 bot
4. 复制 bot token
5. 启用 "Message Content Intent"

#### 2. 邀请 Bot 到服务器

使用 OAuth2 URL：
```
https://discord.com/api/oauth2/authorize?client_id=YOUR_CLIENT_ID&permissions=2048&scope=bot
```

#### 3. 获取 Guild ID 和 User ID

- 启用 Discord 开发者模式（设置 → 高级 → 开发者模式）
- 右键服务器 → 复制 ID（Guild ID）
- 右键你的用户名 → 复制 ID（User ID）

#### 4. 配置 ZeroClaw

```bash
cat >> ~/.zeroclaw/config.toml <<'EOF'

[channels_config.discord]
bot_token = "YOUR_BOT_TOKEN"
guild_id = "YOUR_GUILD_ID"
allowed_users = ["YOUR_USER_ID"]
listen_to_bots = false
mention_only = false
EOF
```

---

## 快速测试配置

### 检查配置

```bash
# 查看当前配置
cat ~/.zeroclaw/config.toml | grep -A 5 "channels_config"

# 测试 channel 健康状态
zeroclaw channel doctor
```

### 启动 daemon

```bash
# 前台运行（查看日志）
zeroclaw daemon

# 或后台运行
zeroclaw service install
zeroclaw service start
```

### 查看状态

```bash
zeroclaw status
```

---

## 配置示例（完整）

```toml
# ~/.zeroclaw/config.toml

default_provider = "kiro"
default_model = "claude-3-5-sonnet"

[channels_config.telegram]
bot_token = "123456:ABC-DEF..."
allowed_users = ["your_username"]
mention_only = false

[memory]
backend = "sqlite"
auto_save = true

[autonomy]
level = "supervised"
workspace_only = true
```

---

## 故障排除

### 问题：Bot 不回复

**检查**：
```bash
# 查看日志
RUST_LOG=debug zeroclaw daemon

# 检查 channel 健康
zeroclaw channel doctor
```

**常见原因**：
1. `allowed_users` 配置错误
2. Bot token 无效
3. 没有启动 daemon

### 问题：权限错误

**Telegram**：
- 确认 `allowed_users` 中的用户名正确（不带 `@`）
- 或使用 `["*"]` 临时允许所有用户

**Discord**：
- 确认 bot 有 "Read Messages" 和 "Send Messages" 权限
- 确认 "Message Content Intent" 已启用

---

## 推荐配置

**开发/测试**：
```toml
[channels_config.telegram]
bot_token = "your-token"
allowed_users = ["*"]  # 允许所有用户（仅测试）
```

**生产环境**：
```toml
[channels_config.telegram]
bot_token = "your-token"
allowed_users = ["your_username", "teammate_username"]  # 白名单
mention_only = true  # 需要 @mention 才响应
```

---

## 下一步

配置完成后：

1. **启动 daemon**：`zeroclaw daemon`
2. **发送测试消息**：在 Telegram/Discord 中发送 "Hello"
3. **查看响应**：Bot 会通过 Kiro Provider 回复
4. **查看日志**：`zeroclaw status` 或 `RUST_LOG=info zeroclaw daemon`

现在你的 ZeroClaw 可以通过 Telegram/Discord 使用 Kiro Provider 了！🎉
