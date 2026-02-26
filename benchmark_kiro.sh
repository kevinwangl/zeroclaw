#!/bin/bash
# Kiro Provider 性能基准测试

set -e

ZEROCLAW_BIN="${ZEROCLAW_BIN:-./target/release/zeroclaw}"
TEST_MESSAGE="Hello, this is a performance test"
ITERATIONS=5

echo "🔬 Kiro Provider 性能基准测试"
echo "================================"
echo ""
echo "配置："
echo "  ZeroClaw: $ZEROCLAW_BIN"
echo "  测试消息: $TEST_MESSAGE"
echo "  迭代次数: $ITERATIONS"
echo ""

# 确保 ZeroClaw 已编译
if [ ! -f "$ZEROCLAW_BIN" ]; then
    echo "⚠️  未找到 ZeroClaw 二进制文件，正在编译..."
    cargo build --release
fi

# 测试函数
run_test() {
    local mode=$1
    local description=$2
    
    echo "📊 测试: $description"
    echo "---"
    
    local total_time=0
    local times=()
    
    for i in $(seq 1 $ITERATIONS); do
        local start=$(date +%s%N)
        $ZEROCLAW_BIN agent --provider kiro -m "$TEST_MESSAGE" > /dev/null 2>&1
        local end=$(date +%s%N)
        
        local elapsed=$(( (end - start) / 1000000 ))  # 转换为毫秒
        times+=($elapsed)
        total_time=$((total_time + elapsed))
        
        echo "  迭代 $i: ${elapsed}ms"
    done
    
    local avg=$((total_time / ITERATIONS))
    
    # 计算中位数
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}"))
    local median=${sorted[$((ITERATIONS / 2))]}
    
    echo ""
    echo "  平均延迟: ${avg}ms"
    echo "  中位延迟: ${median}ms"
    echo "  总耗时: ${total_time}ms"
    echo ""
}

# 清理旧的 daemon 进程
cleanup_daemon() {
    pkill -f "kiro-cli daemon" 2>/dev/null || true
    sleep 1
}

# 测试 1：Oneshot 模式
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 1: Oneshot 模式（每次启动新进程）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cleanup_daemon
export KIRO_USE_DAEMON=false
run_test "oneshot" "Oneshot 模式"

# 测试 2：Daemon 模式（冷启动）
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 2: Daemon 模式（包含冷启动）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cleanup_daemon
export KIRO_USE_DAEMON=true
run_test "daemon-cold" "Daemon 模式（冷启动）"

# 测试 3：Daemon 模式（热启动）
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 3: Daemon 模式（热启动，daemon 已运行）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Daemon 已经在测试 2 中启动，直接测试
export KIRO_USE_DAEMON=true
run_test "daemon-hot" "Daemon 模式（热启动）"

# 测试 4：并发性能
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 4: 并发性能（10 个并发请求）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

export KIRO_USE_DAEMON=true
echo "📊 测试: 并发请求"
echo "---"

local start=$(date +%s%N)
for i in {1..10}; do
    $ZEROCLAW_BIN agent --provider kiro -m "Concurrent test $i" > /dev/null 2>&1 &
done
wait
local end=$(date +%s%N)

local elapsed=$(( (end - start) / 1000000 ))
echo "  10 个并发请求总耗时: ${elapsed}ms"
echo "  平均每个请求: $((elapsed / 10))ms"
echo ""

# 清理
cleanup_daemon

# 总结
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📈 性能总结"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "建议："
echo "  • 生产环境使用 Daemon 模式（KIRO_USE_DAEMON=true）"
echo "  • 预期性能提升：90-95% 延迟降低"
echo "  • 适合高频调用场景（Telegram/Discord channels）"
echo ""
echo "下一步："
echo "  • 查看详细文档: docs/kiro-provider-performance.md"
echo "  • 启用流式响应以进一步提升用户体验"
echo "  • 监控 daemon 内存占用: ps aux | grep 'kiro-cli daemon'"
echo ""
