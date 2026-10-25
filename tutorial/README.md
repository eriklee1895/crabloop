# CrabLoop：从零实现 coding agent Harness

当前稳定的 v1 教程在 **[milestone/v1/tutorial](https://github.com/eriklee1895/crabloop/tree/milestone/v1/tutorial)**，包含 00–06 章、12 张图和六个 additive 代码补丁。

从一次请求出发，依次接上文件工具、Bash、反馈循环，再让同一进程里的下一条任务保留历史。每章通过具体实验解释新增代码，最终得到同一份 v1 实现。

- [开始 v1 学习](https://github.com/eriklee1895/crabloop/tree/milestone/v1/tutorial)：按章节阅读和跟做。
- [v1 完整增量](https://github.com/eriklee1895/crabloop/tree/milestone/v1/tutorial/v1/patches)：从空目录重建。
- [后续演进建议](../docs/git-workflow.md)：main 从 v1 GA 分批增加 context 与 compaction，验收后再建立 milestone/v2。

main 当前仍保留早期交互与 context 实验；目录内的旧教程可作历史参考，跟做 v1 时使用上面的稳定分支。main 的基线整理尚未执行。

旧 [v2 标签](https://github.com/eriklee1895/crabloop/tree/v2)保留原实现。新的 v2 不提前建立学习分支，也不直接复用旧版的完整 CLI 与状态结构。
