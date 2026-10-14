# CrabLoop：从零实现 coding agent Harness

一个 Rust 项目的测试失败了。我们希望对程序说：“找到问题，修好它，再运行测试。”

Harness 负责接通模型、文件和 Bash 工具，再把执行结果送回模型。我们按请求、工具、反馈循环和连续对话的顺序逐步实现：

```mermaid
flowchart LR
    A["01–03<br/>模型与工具"] --> B["04–05<br/>反馈循环与任务"]
    B --> C["06<br/>连续对话"]
```

## 从哪里开始

先读 [00：看清一次修复](v1/00-state-map.md)，认出模型、harness 和工具各自做什么，再按下面的顺序跟做。你需要能阅读基本 Rust 代码；遇到 `Result`、借用或异步等待时，正文会结合当时的代码解释。

| 章节 | 这一章亲手看见什么 |
|---|---|
| [01：把第一条请求送出去](v1/01-responses.md) | 请求里的任务，怎样变成响应里的文本或工具调用 |
| [02：让一个工具调用真正读到文件](v1/02-file-tools.md) | 文件确实被读取；匹配不明确时，编辑被拒绝 |
| [03：让测试作为一个真实进程运行](v1/03-bash.md) | 输出、退出码与超时怎样成为可检查的结果 |
| [04：把结果送回去，循环就开始了](v1/04-agent-loop.md) | 历史从一项长成五项，工具错误也能回到模型手里 |
| [05：让四个工具一起做完一件事](v1/05-v1-coding-task.md) | 修好一个函数、整理一份报告，并在循环之外验证 |
| [06：让一个任务接着另一个任务](v1/06-conversation.md) | 第二次提问携带第一次的回答和工具结果 |

## 一份源码，两种学习方式

**完整的 v1 学习代码是 `milestone/v1`。** 想先看成品，可以按 [v1 README](https://github.com/eriklee1895/crabloop/blob/milestone/v1/README.md) 启动，再从该分支的 `src/main.rs` 和 `src/lib.rs` 的 `run_turn` 追踪一次任务。

想从空目录跟做，就按[七个完整代码增量](v1/patches/README.md)建立练习工程。正文解释每一步为什么出现，补丁补齐导入、类型、入口与测试；可以手动参照 diff 编写，也可以应用补丁核对结果。不要把练习补丁直接应用到已经完整的参考仓库。

章节中的 `step-*` 是这一阶段的代码书签，方便查阅当时的接口。前几章先学一次任务内部的循环；第 05 章把代码收束到当前 v1，第 06 章再拆开其中的连续对话。最后得到的源码与 `milestone/v1` 相同。

当前 v1 在终端里支持连续提问，同一进程保留历史，Ctrl-D 退出；管道输入仍作为一次任务处理。历史只存在内存中，暂时没有跨进程恢复或摘要压缩。

## 继续学习 context 与 compaction

完成 v1 后，从 [v2：06 context 投影](v2/06-context-projection.md)开始，再读 [07 compaction](v2/07-compaction.md)、[08 对照实验](v2/08-context-experiments.md)和 [09 复盘](v2/09-v2-retrospective.md)。按各章指定的代码基线继续，观察历史变长以后，下一轮应该保留什么。

main 的完整实现同时包含交互与 compaction，源码入口是 `Session::turn`。想观察实际执行过程，可以选读[调试指南](../docs/debugging.md)；[Session 与输入编辑器](v1/05b-interactive.md)保留为基于历史提交的选读专题。

v1 的 06 章讲最小连续对话，v2 的 06 章讲 context 投影；它们属于不同阶段。版本关系见[Git 工作流](../docs/git-workflow.md)。
