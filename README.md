# CrabLoop 🦀

用 Rust 从零写一个能读文件、修改代码、运行测试，再根据真实反馈继续行动的 mini agent。

这是 **milestone/v1 的最小教学实现**：用 **300+ lines** 理解模型、工具、反馈与停止组成的 agent loop。

## 先运行一个任务

需要 Rust 1.94.0 和 Bash。在当前目录的 .env 配置 DEEPSEEK_API_KEY，然后：

```bash
cargo run
```

终端会显示以下提示：

```text
🦀 CrabLoop · a small Rust agent
Enter one task. Ctrl-D to exit.

crab>
```

在 `crab>` 后输入一个任务，按 Enter 提交；完成后程序退出。脚本和多行任务也从 stdin 传入：

```bash
printf '%s\n' "检查当前目录" | cargo run --quiet
```

v1 只从 stdin 接收任务，不接受命令行参数；空输入或 EOF 直接退出。

工作区就是启动目录。要操作另一个目录，先构建，再从那个目录调用二进制：

```bash
cargo build --locked
crabloop_bin="$PWD/target/debug/crabloop"
cd /path/to/project
printf '%s\n' "读取项目说明并检查测试" | "$crabloop_bin"
```

默认使用 DeepSeek，配置从启动目录的 `.env` 和进程环境读取，进程环境优先。OpenAI 或兼容端点的配置见 [.env.example](.env.example)。不要提交凭证。

运行时会显示工具名称、命令或路径、执行状态和输出预览。需要完整 JSON 调试轨迹时，设置 `CRABLOOP_TRACE_TOOLS=1`。

## 从哪里读核心

从 [src/lib.rs](src/lib.rs) 的 run 函数开始。input 保存当前任务的完整历史：请求模型、追加 response.output、执行工具、用原 call_id 追加结果，再进入下一轮。模型给出最终回答或耗尽预算时停止。

| 文件 | 职责 |
|---|---|
| src/main.rs | 调用入口、显示最终答案和退出状态 |
| src/cli.rs | 单次任务输入和最小配置 |
| src/lib.rs | 限制、停止原因与手写 agent loop |
| src/model.rs | Responses 请求和完整响应校验 |
| src/tools.rs | read_file、write_file、edit_file、bash |

一个任务可以触发多轮模型和工具交互，工具失败也会反馈给模型。模型说完成后，仍需独立检查文件和测试结果。v1 每次运行只处理一个任务。

## 教程

从[教程入口](tutorial/README.md)开始，也可以用[七个增量](tutorial/v1/patches/README.md)从空目录重建这份 v1 实现。
