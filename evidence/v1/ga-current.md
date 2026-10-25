# 当前 v1 GA 复核

日期：2026-10-25。结论：**符合既定教学 scope，可以 GA。** 本文评价四工具、原生 Responses、手写 loop、同进程连续任务与配套教程，不把后续版本能力列为 v1 缺陷。

运行基线为合并 PR #1 后的 `9a8077e`，生产 Rust 为 324 code / 370 physical lines。每份运行记录都保存相同的源码 SHA256；后续文档提交不改变这些运行代码。

## 实际验证

| 检查 | 结果与证据 |
|---|---|
| 离线基线 | fmt、clippy、33 项 Rust、6 项 Python 测试通过；覆盖 ID/文本校验、工具错误反馈、精确编辑、预算与停止、Bash 输出及超时 |
| 教程重建 | 六个 additive 补丁逐阶段 fmt/tests 通过，最终文件与实现一致；[PR 复核记录](../../docs/devlog/39-v1-slim-review.md) |
| DeepSeek 修复任务 | 修复前 verifier 退出 101，修复后退出 0；原 tests.rs 未改，NOTES.md 已生成；[运行记录](20261025T131405711975Z-coding-deepseek.json) |
| DeepSeek 资料整理 | 初始缺少产物时 verifier 退出 1，生成报告后退出 0；事实、统计、来源和原始文档核对通过；[运行记录](20261025T131422650777Z-report-deepseek.json) |
| DeepSeek 连续任务 | 第二个任务只指“刚才的文件”，实际只把第二行 eggs 改为 oatmeal，第一与第三行字节保持不变；[运行记录](20261025T131808448010Z-ga-conversation.json) |
| 本地服务 + 真实 PTY | 两次输入之间保留回答，空行不发送请求，提示符再次出现，Ctrl-D 以 0 退出；见同一记录的 pty 部分 |

真实模型使用 `deepseek-flash`。修复、报告和连续任务分别由文件、独立 verifier、前后快照验证，不以模型的最终措辞作为成功依据。PTY 检查使用本地服务，它证明终端与历史通路，不代表真实模型的语义表现。

## 已知 scope

每个非空 stdin 行是一个任务；使用三个环境变量配置服务，不提供完整 CLI、`.env` 自动读取、provider presets、输入编辑、compaction、持久化或恢复。Bash 没有 sandbox；当前仅移除子进程的 OPENAI_API_KEY，不能据此声称完整隔离。

旧 `v1`/`step-*` 标签及早期验收仍保留其历史含义，本次不移动它们。当前唯一 v1 学习入口继续使用 `milestone/v1`。

## 版本演进

`milestone/v2` 已删除；原提交 `5a5e96e` 仍由不可变 `v2` 标签保留。下一阶段建议将 main 内容明确对齐 v1 GA，保留旧 main 历史，再分批加入 context/compaction。新的 v2 在 main 验证满意后才建立 milestone/v2；现有历史 v2 不作为这条新路线已经完成的证明。

原始日志与生成工作区保留在本地，仓库只提交选择过的运行记录与结论。没有新建或移动发布标签。
