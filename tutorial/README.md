# CrabLoop：从零实现 coding agent Harness

一个 Rust 项目的测试失败了。我们希望对程序说：“找到问题，修好它，再运行测试。”

要完成这件事，Harness 需要把几个环节接起来：让模型提出动作，执行文件和 Bash 工具，再把结果送回模型。测试失败后，下一次请求带上错误，模型才能据此调整。

我们从一个空 Cargo 工程开始，先发出一次 Responses 请求，再依次接入文件工具、Bash 和 agent loop。每一步都运行程序，观察请求、结果和状态怎样变化。

v2 继续使用这份实现，学习 context 管理与 compaction：历史变长后，下一轮保留什么，摘要有没有丢掉关键事实。

## 怎样沿着这条路线学习

v1 围绕模型交互、四个工具和行动循环展开，核心实现目标约 300 行。v2 在相同代码上增加上下文管理与摘要压缩，总目标约 500 行。源码始终位于仓库根目录的 Cargo 工程中。

v1 的学习代码统一为 `milestone/v1`。直接阅读该分支，或按[七个完整增量](v1/patches/README.md)从空目录重建。每章说明代码变化、运行方法和观察结果；章节基线以各章标注的 commit/tag 为准。

从 [00 状态地图](v1/00-state-map.md) 开始，再阅读 [01 第一次请求](v1/01-responses.md)、[02 文件工具](v1/02-file-tools.md)、[03 Bash](v1/03-bash.md)、[04 行动循环](v1/04-agent-loop.md) 和 [05 综合任务](v1/05-v1-coding-task.md)。

当前 v1 支持终端连续提问，进程内保留历史；管道输入只运行一次。从 src/main.rs 看历史的生命周期，再从 src/lib.rs 的 run_turn 追踪模型与工具循环。

[交互补篇](v1/06-conversation.md)保存使用 Session 和输入编辑器的历史实验。v2 各章按指定基线学习 context 管理与 compaction。
