# CrabLoop 🦀

用 Rust 从零写一个能读文件、修改代码、运行测试，再根据真实反馈继续行动的 mini coding agent。

v1 用 **300+ lines** 演示一个完整的 agent loop。

## Quick Start

配置三个环境变量。以 DeepSeek 为例；使用 OpenAI 时省略 `OPENAI_BASE_URL`，模型换成账号可用的 Responses 模型：

```bash
export OPENAI_API_KEY=你的密钥
export OPENAI_BASE_URL=https://api.deepseek.com
export CRABLOOP_MODEL=deepseek-flash
cargo run
```

在 `crab>` 后输入任务，按 Enter 提交，例如：

```text
crab> 读取 README.md，概括项目结构
```

每个非空行是一个任务，同一进程保留前面的对话；Ctrl-D 退出。管道输入走同一条路，每行一轮：

```bash
printf '%s\n' "检查当前目录" "哪个文件最大？" | cargo run --quiet
```

启动目录就是工作目录。运行时 stderr 显示每轮工具调用数和 `→ 工具名 参数 · ok=...`，stdout 只有最终回答。

工具会直接读写文件和执行 Bash；v1 没有 sandbox，bash 子进程仍继承除 `OPENAI_API_KEY` 外的环境变量。

## 从哪里读核心

从 [src/lib.rs](src/lib.rs) 的 `run_turn` 函数开始，沿着“请求模型 → 执行工具 → 回传结果 → 继续或停止”阅读手写循环。模型交互使用 OpenAI Responses API。

| 文件 | 职责 |
|---|---|
| [src/main.rs](src/main.rs) | 逐行读取任务、持有对话历史、打印回答和退出状态 |
| [src/lib.rs](src/lib.rs) | `Config`、停止原因 `Stop` 与手写 agent loop `run_turn` |
| [src/model.rs](src/model.rs) | Responses 请求和完整响应校验 |
| [src/tools.rs](src/tools.rs) | `read_file`、`write_file`、`edit_file`、`bash` |

工具失败会反馈给模型，供下一轮调整行动。模型说完成后，仍需检查文件和测试结果。

## 教程

从[教程入口](tutorial/README.md)开始，也可以用[六个增量](tutorial/v1/patches/README.md)从空目录重建这份 v1 实现。
