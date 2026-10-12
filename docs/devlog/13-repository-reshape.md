# 13：整理学习入口与临时分支

> 后续整理：用户已将交互增量统一到 milestone/v1，本文的 v1.1 是当时名称；对应分支和标签已移除，见 [统一 v1 记录](14-unify-v1.md)。
日期：2026-10-12。用户同意按轻量 reshape 方案整理仓库，同时确认早期版本是否已经包含使用优化。

先对比 v1.1 和 main 的 src/cli.rs、src/terminal.rs，两份文件完全一致。v1.1 已支持默认工作目录、当前目录 .env、-p、输入编辑与多轮聊天；它仍保留 v1 的完整内存历史，没有 v2 的压缩或 journal。原始 v1、v2 是固定历史快照。

README 增加 main / v1 / v1.1 / v2 的入口对照，建议日常体验使用 main，学习早期交互切到 milestone/v1.1。教程入口明确了阅读次序与代码基线：可以先学交互再学压缩；Git 中 v1.1 与 v2 分别从 v1 演进。Git 工作流和 AGENTS.md 同步说明长期分支与清理规则。

刷新远端后，确认以下临时分支 tip 都在 main 历史中，清理对应本地名称与存在的远端名称：

| 临时分支 | 原 tip | 清理范围 |
|---|---|---|
| docs/codex-tui-reference | 2e65c79 | 本地、远端 |
| docs/learning-readability | 833737b | 本地、远端 |
| feat/crabloop-v1-v2 | b8e3c7b | 本地、远端 |
| feat/interactive-chat | 9398117 | 本地 |
| fix/simple-startup | f66a777 | 本地、远端 |

codex/v1-interactive 指向 9c5cb11，与已发布的 milestone/v1.1 和 v1.1 标签对应提交一致。确认其 worktree 干净后转为 detached v1.1，删除重复本地分支，并向应用提交 worktree 归档请求；分支清理不依赖异步归档是否已经完成。远端删除按预期 tip 带 lease 原子提交，若远端在检查后发生变化就拒绝删除。

长期分支保持 main、milestone/v1、milestone/v1.1、milestone/v2。全部 18 个受保护引用（3 个里程碑分支、15 个标签）逐一与整理前比较，指向不变；旧提交、章节补丁与验收记录继续可访问。本次只修改导航和开发说明，没有改写代码或提交历史。

检查：cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings、61 Rust 测试和 6 Python 测试通过；修改文档的本地链接、AGENTS.md 的 8,192 字节上限及 git diff --check 均通过。
