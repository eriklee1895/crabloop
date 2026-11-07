# v1 回到可一眼追踪的核心

日期：2026-10-12。用户确认 v1 应帮助初学者快速看见 agent loop，目标约 300 行、全部生产 Rust 严格 <400 行。

从 571 行的交互实现开始，先让新增行数门禁和“聊天留后续”的回归失败，再做减法。run(Config) 直接持有历史和 client；移除跨提问 Session、Progress、终端编辑器、聊天命令和取消状态。四工具、完整 Responses output、call_id 关联、错误反馈、输入/输出预算和工具超时保持。

正常格式化后首稿仍为 416 行，因此进一步收回 CLI 范围：工作目录固定为 cwd，请求预算固定为 131,072 字节，provider 显式选择且默认 DeepSeek；没有模型名/hostname/Key 猜测。保留 .env 数据读取、-p/--print、无参数单次输入和管道。空输入不论来自参数或 stdin 都直接结束，不请求模型；这条规则先观察回归失败再实现。删去 v1 未使用的 usage 字段，没有修改模型协议或工具行为。

最后为 398 代码行 / 403 物理行。差异来自减少功能和状态，没有压缩 rustfmt 或把运行逻辑移到脚本。src/model.rs 和 src/tools.rs 保持展开的协议结构，核心 loop 在 src/lib.rs 的一个 run 函数中。行数门禁：python3 scripts/count-lines.py --max 399。

当前 40 Rust + 4 Python 测试、fmt/clippy 与行数检查通过。被移出的跨用户回合功能及其测试继续保留在 main。真实 DeepSeek 修复已通过：commit be8131b，9 次工具调用、11.21 秒；修复前失败、修复后测试通过，独立 verifier 通过，原测试未改且 NOTES.md 生成。证据见 [本次运行](../../evidence/v1/20261012T035314137932Z-coding-deepseek.json)。实际 PTY 加本地模型 fixture 也验证了无参数中文任务输入、一次请求与正常退出。最终审查未发现运行时阻塞；两项清理已落实：第 01 章显式给出历史 checkout，v1 live driver 拒绝 --context-bytes 和 --stage v2，防止产生错误的预算/版本证据。后者先观察回归失败再修复；原始标签和旧模型证据不改写。
