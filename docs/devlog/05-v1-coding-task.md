# 05：v1 实际任务与规模收束

日期：2026-10-11。里程碑：v1。

修复 fixture 初始失败，独立 verifier 也失败。scripted_bugfix_loop 在本地 HTTP stub 指引下实际执行四个工具，完成失败测试、读写、修改与通过测试；它证明组合机制，真实 provider 证据另行保存。

DeepSeek 主模型和 TeamoRouter gpt-6-luna 均完成真实修复，保留原测试并通过四项独立测试。DeepSeek 同时完成本地资料报告。报告首轮被 verifier 误拒：counts 多了正确的 total=3。新增三个 verifier 回归用例后，按必需事实投影检查、允许合法扩展字段，同一产物复核通过；保留原失败记录，不改写为从未失败。

核心初稿为 407 行。移除重复持有 Config 字段的 ModelClient，以及只包装 JSON 的 ToolResult 后为 367 行。统一 100 列 rustfmt、use_small_heuristics=Max 后为 314 代码行、319 物理行。后一个下降来自格式，不能算逻辑减少。未把运行时代码挪入脚本规避计数。用户后续明确行数可弹性浮动，不继续压缩代码。

最终接口是 model::request/respond 两个函数，run 持有复用的 HTTP client，tools::execute 返回 JSON，call_id 的 Responses 包装由 loop 负责。前四个代码书签保留原接口；v1 保存本次整理。

本阶段验证：29 项 Rust 集成测试，3 项 Python verifier 回归测试；fmt、clippy 通过。真实结果和已知限制见[验收表](../../evidence/v1/acceptance.md)。

同时补充第 01 章的完整空项目探针，读者能从 cargo new 发出第一次请求，再对照固定书签逐步加上协议校验、工具与循环。后续路线已调整为 v3 配置/模型目录，v4 Skills/联网，v5 会话恢复；这些没有提前混入 v1。
