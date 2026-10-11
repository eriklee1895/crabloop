# v2 验收记录

日期：2026-10-11。核心 504 代码行，511 物理行。一个根 Cargo 工程，所有生产 Rust 均计入；测试、教程、fixtures 和开发脚本单独保存。

| 层次 | 本次结果 |
|---|---|
| Rust 确定性测试 | 46 项：9 协议、13 工具、7 loop、8 context、9 compaction |
| Python verifier | 3 项回归通过 |
| 检查 | cargo fmt --check、clippy --all-targets -D warnings、cargo test --locked 通过 |
| 强制压缩真实任务 | deepseek-flash，10240-byte hard / 7680-byte soft；11 主轮，5 次成功压缩，Final，agent exit=0 |
| 任务产物 | 原测试先失败后通过，独立 verifier 4/4，tests.rs 未改，NOTES.md 已生成 |
| 固定上下文 | 每个主请求原任务与根 AGENTS.md 均保留 |
| 默认预算对照 | 相同最终 Rust 源码与任务，49152-byte，0 次压缩，任务通过 |
| 输入保留对照 | full 6989 / tail 1577 / summary+tail 1851 bytes；tail 丢失预设事实 |
| 人工摘要复核 | 主要测试状态与剩余工作保留；首份摘要有 AGENTS 读取状态矛盾与来源误归，未称为无损 |

[小预算真实运行](20261011T143500013361Z-coding-deepseek.json)：25.63 秒，五次压缩为 8408→7346、8497→6046、9071→5993、8972→5808、7703→6253 bytes。[默认预算对照](20261011T143557216175Z-coding-deepseek.json)：9.92 秒。输入轨迹不同、样本很少，不用于宣称压缩更快、更便宜或模型稳定性能。

[source_commit/source_sha256] 的含义沿用 v1：运行时工作区含未提交变化，source_sha256 标识实际源文件；source_commit 是当时最近提交。真实记录保留 main/summary usage。原始请求/响应和本机轨迹保存在忽略的 .crabloop 及 playground 内。

失败尝试未删除：8 KB 无可整理前缀、摘要将历史当继续动作、摘要推理耗尽生成额度、候选仍超阈值。16 KB 的一次运行修复成功但没有压缩，passed=false 是该次 require-compaction 条件未满足；不代表函数仍错误。部分错误发生时源码已修好，整体运行仍单独标为未完成。

摘要请求使用 reasoning.effort=none、至少 4096 输出 tokens，最终摘要仍受 min(4096, hard/8) 字节限制。DeepSeek 此路径已实测；gpt-6-luna 在 v1 通过真实工具任务，v2 effort 支持仅核对官方文档，没有把它算作 v2 真实验收。

限制：摘要栏目检查不验证语义；byte 预算不是 tokenizer；默认推理主调用仍可能 incomplete；JSONL 仅记录单次运行，不提供恢复；Bash 仅回收直接子进程；路径约定与输出限额不等于 sandbox。v3–v5 的 home/config/catalog、Skills/web、sessions 仍是规划。
