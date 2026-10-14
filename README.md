# CrabLoop 🦀

用 Rust 从零写一个能读文件、修改代码、运行测试，再根据真实反馈继续行动的 mini agent。

v1 用 **300+ lines** 演示一个完整的 agent loop。

## Quick Start

默认使用 DeepSeek。在当前目录的 `.env` 中配置 `DEEPSEEK_API_KEY`，其他模型配置见 [.env.example](.env.example)。然后运行：

```bash
cargo run
```

在 `crab>` 后输入任务，按 Enter 提交，例如：

```text
crab> 读取 README.md，概括项目结构
```

每次运行处理一个任务，完成后退出。任务也可以通过管道传入：

```bash
printf '%s\n' "检查当前目录" | cargo run --quiet
```

任务从 stdin 读取，不接受命令行参数。

配置读取启动目录的 `.env`，进程环境优先。

运行时会显示工具名称、命令或路径、执行状态和输出预览。

## 从哪里读核心

从 [src/lib.rs](src/lib.rs) 的 `run` 函数开始，沿着“请求模型 → 执行工具 → 回传结果 → 继续或停止”阅读手写循环。模型交互使用 OpenAI Responses API。

| 文件 | 职责 |
|---|---|
| [src/main.rs](src/main.rs) | 调用入口、显示最终答案和退出状态 |
| [src/cli.rs](src/cli.rs) | 单次任务输入和最小配置 |
| [src/lib.rs](src/lib.rs) | 限制、停止原因与手写 agent loop |
| [src/model.rs](src/model.rs) | Responses 请求和完整响应校验 |
| [src/tools.rs](src/tools.rs) | `read_file`、`write_file`、`edit_file`、`bash` |

工具失败会反馈给模型，供下一轮调整行动。模型说完成后，仍需检查文件和测试结果。

## 教程

从[教程入口](tutorial/README.md)开始，也可以用[七个增量](tutorial/v1/patches/README.md)从空目录重建这份 v1 实现。
