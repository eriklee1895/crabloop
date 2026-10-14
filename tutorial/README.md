# CrabLoop：从零实现 coding agent Harness

一个 Rust 项目的测试失败了。我们希望对程序说：“找到问题，修好它，再运行测试。”

要完成这件事，Harness 需要把几个环节接起来：让模型提出动作，执行文件和 Bash 工具，再把结果送回模型。测试失败后，下一次请求带上错误，模型才能据此调整。

我们从一个空 Cargo 工程开始，先发出一次 Responses 请求，再依次接入文件工具、Bash 和 agent loop。每一步都运行程序，观察请求、结果和状态怎样变化。

v2 继续使用这份实现，学习 context 管理与 compaction：历史变长后，下一轮保留什么，摘要有没有丢掉关键事实。

## 怎样沿着这条路线学习

v1 围绕模型交互、四个工具和行动循环展开，核心实现目标约 300 行。v2 在相同代码上增加上下文管理与摘要压缩，总目标约 500 行。源码始终位于仓库根目录的 Cargo 工程中。

v1 的学习代码统一为 `milestone/v1`。直接阅读该分支，或按[七个完整增量](v1/patches/README.md)从空目录重建。每章说明代码变化、运行方法和观察结果；章节基线以各章标注的 commit/tag 为准。

从 [00 状态地图](v1/00-state-map.md) 开始，再阅读 [01 第一次请求](v1/01-responses.md)、[02 文件工具](v1/02-file-tools.md)、[03 Bash](v1/03-bash.md)、[04 行动循环](v1/04-agent-loop.md) 和 [05 综合任务](v1/05-v1-coding-task.md)。

| 阶段 | 章节范围 | 主要问题 |
|---|---|---|
| tutorial/v1/ | 00–05：状态地图、Responses、文件工具、Bash、循环、综合修复 | 一条请求怎样变成真实的修改和测试？ |
| tutorial/v1/05b-interactive.md | 后续选读：多轮用户 Session，基于历史交互提交 | 下一句怎样接着前面的历史继续？ |
| tutorial/v2/ | 06–09：上下文投影、摘要、对照实验、综合复盘 | 任务继续推进时，模型应该看到哪些历史？ |

详细步骤见[实施与学习计划](../docs/plan.md)，版本边界见[设计说明](../docs/design.md)。实际开发经过从[仓库起步记录](../docs/devlog/00-bootstrap.md)开始。

想观察执行中的状态，可以在 main 选读[调试指南](../docs/debugging.md)：先调试固定响应测试，再对照 JSONL 和 HTTP 请求。它是独立的调试实践，不纳入 v1 学习分支。

完成 v1 后，先读 [06 上下文投影](v2/06-context-projection.md)、[07 摘要压缩](v2/07-compaction.md)、[08 对照实验](v2/08-context-experiments.md) 和 [09 复盘](v2/09-v2-retrospective.md)，使用每章指定的历史基线。

当前 milestone/v1 支持最小连续对话；[05b 接着问下一句](v1/05b-interactive.md)以历史提交 9c5cb11 讲解 Session 与输入编辑器。main 同时包含完整交互与 compaction，版本关系见[Git 工作流](../docs/git-workflow.md)。
