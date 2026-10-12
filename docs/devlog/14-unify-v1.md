# 14：使用优化统一归入 v1

日期：2026-10-12。用户要求不再保留单独的 v1.1，直接保留一个 v1 学习分支。

milestone/v1 从 a570154 快进到 b9226d6，完整包含先前交互支线的提交，以及本次名称和文档整理。现在 `git switch milestone/v1` 后直接 `cargo run` 就进入多轮聊天，-p 单次任务、默认工作目录和 .env 读取也在这一分支。生产 Rust 与原交互提交 9c5cb11 完全一致，仍为 571 代码行 / 581 物理行，无 v2 压缩或 journal。

复核旧支线时发现，三个 CLI 协议测试没有设置启动目录；在带有 .env 的主工作区运行时，其中“缺少 Key”测试失败。补上与 main 相同的临时 current_dir，避免读到开发者配置。修改后 43 项 Rust、3 项 Python 回归、fmt 和 clippy 全部通过。

main 的 README、教程入口、交互补篇、路线图、Git 工作流和 AGENTS.md 统一指向 milestone/v1。旧开发记录保留当时经过，并加注名称已经调整。main 运行代码没有变化，61 项 Rust、6 项 Python 测试及 fmt/clippy 通过。

发布时同时更新 main 和 milestone/v1，删除远端 milestone/v1.1 与 v1.1 标签，再清理本地同名引用。旧交互提交继续是 milestone/v1 的祖先。原始 v1 / step-05-v1 标签保留 314 行最小循环，v2 和所有其余 step 标签保持原值，因此章节补丁和旧验收基线不变。

长期分支为 main、milestone/v1、milestone/v2。学习分支可以吸收同阶段的使用优化，历史标签固定原验收快照。以后不再为同阶段的入口改进增加一个学习版本。
