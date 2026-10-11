# 00：仓库、目录与教程的起点

日期：2026-10-11。

CrabLoop 的目标已经收敛为两个阶段：先用约 300 行 Rust 完成四个工具和 coding 闭环，再扩展到约 500 行，练习上下文管理与摘要压缩。模型接入选择 OpenAI 原生 Responses API。

公开仓库位于 [eriklee1895/crabloop](https://github.com/eriklee1895/crabloop)。初始提交 [86b3f24](https://github.com/eriklee1895/crabloop/commit/86b3f24030451197a65ed548bb6da91d8697c27f) 包含 README、AGENTS.md、.gitignore，以及两版设计和计划。

## 目录怎样确定

Cargo 工程位于根目录，维护一份持续演进的源码。教程放在 tutorial/v1/、tutorial/v2/；开发经过放在 docs/devlog/。章节与代码通过提交关联，完整阶段使用 v1、v2 tag。

最初计划中的章节路径已统一到 tutorial/。设计和计划分别归到 docs/design.md 与 docs/plan.md，文档目录直接表达内容用途。

## 这次明确的写法

教程要从零开始，耐心讲解一个最小 harness 怎样逐步长出来。每一步先交代正在解决的问题，再写一小段代码，运行、观察，并说明下一步为什么必要。

这一约定已经写入 [AGENTS.md](../../AGENTS.md) 和[写作指南](../writing-guide.md)。[教程入口](../../tutorial/README.md)提供了开篇样文，后续章节会沿着这个节奏展开。

## 当前验证与下一步

本次变化是目录与文档工作。检查包括 Markdown 相对链接、代码围栏、旧章节路径残留，以及 Git diff 的空白错误。Rust 运行行为和真实模型实验仍由后续任务验证。

接下来从状态地图和第一条 Responses 请求开始，让教程描述的第一个小程序真正运行起来。
