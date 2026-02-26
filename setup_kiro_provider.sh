#!/bin/bash
# 快速设置 Kiro CLI 作为 ZeroClaw Provider

set -e

echo "🦀 ZeroClaw + Kiro CLI 集成设置"
echo ""

# 检查 kiro-cli 是否可用
if ! command -v kiro-cli &> /dev/null; then
    echo "❌ 错误：找不到 kiro-cli 命令"
    echo ""
    echo "请先安装 Kiro CLI 或设置 KIRO_CLI_PATH 环境变量："
    echo "  export KIRO_CLI_PATH=/path/to/kiro-cli"
    exit 1
fi

echo "✅ 找到 Kiro CLI: $(which kiro-cli)"
echo ""

# 编译 ZeroClaw
echo "📦 编译 ZeroClaw（包含 Kiro Provider）..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo "✅ 编译成功"
echo ""

# 配置 ZeroClaw
echo "⚙️  配置 ZeroClaw 使用 Kiro Provider..."

ZEROCLAW_CONFIG="$HOME/.zeroclaw/config.toml"

if [ -f "$ZEROCLAW_CONFIG" ]; then
    echo "⚠️  配置文件已存在: $ZEROCLAW_CONFIG"
    read -p "是否覆盖 default_provider 设置？(y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 备份现有配置
        cp "$ZEROCLAW_CONFIG" "$ZEROCLAW_CONFIG.backup.$(date +%s)"
        echo "✅ 已备份现有配置"
        
        # 更新 provider 设置
        if grep -q "^default_provider" "$ZEROCLAW_CONFIG"; then
            sed -i.tmp 's/^default_provider.*/default_provider = "kiro"/' "$ZEROCLAW_CONFIG"
            rm -f "$ZEROCLAW_CONFIG.tmp"
        else
            echo 'default_provider = "kiro"' >> "$ZEROCLAW_CONFIG"
        fi
        
        if grep -q "^default_model" "$ZEROCLAW_CONFIG"; then
            sed -i.tmp 's/^default_model.*/default_model = "kiro-default"/' "$ZEROCLAW_CONFIG"
            rm -f "$ZEROCLAW_CONFIG.tmp"
        else
            echo 'default_model = "kiro-default"' >> "$ZEROCLAW_CONFIG"
        fi
        
        echo "✅ 已更新配置文件"
    fi
else
    echo "📝 创建新配置文件..."
    mkdir -p "$HOME/.zeroclaw"
    cat > "$ZEROCLAW_CONFIG" <<EOF
default_provider = "kiro"
default_model = "kiro-default"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true

[gateway]
port = 42617
host = "127.0.0.1"

[autonomy]
level = "supervised"
workspace_only = true
EOF
    echo "✅ 已创建配置文件"
fi

echo ""
echo "🎉 设置完成！"
echo ""
echo "快速测试："
echo "  ./target/release/zeroclaw agent -m \"Hello from Kiro!\""
echo ""
echo "交互模式："
echo "  ./target/release/zeroclaw agent"
echo ""
echo "查看状态："
echo "  ./target/release/zeroclaw status"
echo ""
echo "环境变量（可选）："
echo "  export KIRO_CLI_PATH=/custom/path/to/kiro-cli"
echo "  export KIRO_MODEL=your-preferred-model"
echo ""
