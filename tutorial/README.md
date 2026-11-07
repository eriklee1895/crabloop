# CrabLoop：从零实现 coding agent Harness

一个 Rust 项目的测试失败了。我们希望对程序说：“找到问题，修好它，再运行测试。”

Harness 负责接通模型、文件和 Bash 工具，再把执行结果送回模型。我们按请求、工具、反馈循环和连续对话的顺序，从一个空目录开始逐步实现：

```mermaid
flowchart LR
    A["01–03<br/>模型与工具"] --> B["04–05<br/>反馈循环与任务"]
    B --> C["06<br/>连续对话"]
```

## 从哪里开始

先读 [00：看清一次修复](v1/00-state-map.md)，认出模型、harness 和工具各自做什么，再按下面的顺序跟做。你需要能阅读基本 Rust 代码；遇到 `Result`、借用或异步等待时，正文会结合当时的代码解释。

v1 没有 sandbox：接上真实模型后，模型能在工作目录里执行任意 bash 命令，也能读到除 `OPENAI_API_KEY` 之外的环境变量。请在测试目录或 container 里跟做，详见[第 03 章的安全提示](v1/03-bash.md)。

| 章节 | 这一章亲手看见什么 |
|---|---|
| [01：把第一条请求送出去](v1/01-responses.md) | 请求里的任务，怎样变成响应里的文本或工具调用 |
| [02：让一个工具调用真正读到文件](v1/02-file-tools.md) | 文件确实被读取；匹配不明确时，编辑被拒绝 |
| [03：让测试作为一个真实进程运行](v1/03-bash.md) | 输出、退出码与超时怎样成为可检查的结果 |
| [04：把结果送回去，循环就开始了](v1/04-agent-loop.md) | 历史从一项长成五项，工具错误也能回到模型手里 |
| [05：让四个工具一起做完一件事](v1/05-v1-coding-task.md) | 修好一个函数、整理一份报告，并在循环之外验证 |
| [06：让一个任务接着另一个任务](v1/06-conversation.md) | 第二个任务携带第一个任务的回答和工具结果 |

## 一份源码，两种读法

**完整的 v1 代码在 `milestone/v1`**，`src` 下只有四个文件，约 300 行。想先看成品，可以按仓库 [README](../README.md) 启动，再从 `src/lib.rs` 的 `run_turn` 读起。

想从空目录跟做，先按[增量入口](v1/patches/README.md)准备练习目录。第 01–06 章各对应一个补丁，每个补丁只增加代码：第 01 章写下的函数签名，到最后仍然是同一个。你可以对照正文手写，也可以应用补丁再阅读 diff。最后得到的 `src` 与 `milestone/v1` 完全相同。

所有需要真实模型的实验都使用同一组环境变量：

```bash
export OPENAI_API_KEY=你的密钥
export OPENAI_BASE_URL=https://api.deepseek.com   # 使用 OpenAI 时省略
export CRABLOOP_MODEL=deepseek-flash
```

下一阶段计划在 main 从这份 v1 逐步增加 context 与 compaction，验收后再创建 milestone/v2。旧版 [v2 教程](https://github.com/eriklee1895/crabloop/tree/v2/tutorial/v2)保留为历史参考。
