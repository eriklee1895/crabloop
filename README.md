# CrabLoop 🦀

A small coding-agent harness that grows step by step in Rust.

用 Rust 从零实现一个小型 coding agent，通过读取代码、修改文件、运行测试和处理反馈，练习 agent harness 的核心机制。

这是 **v1.1 交互学习快照**：在原 v1 的四工具循环上提前加入多轮终端对话和 `-p` 单次任务。原 `v1` 标签保留 314 行闭环；这里完整实现为 **571 行代码 / 581 物理行**，包含所有 CLI、界面和错误处理。后续主线继续增加上下文压缩、配置和其他能力。

## 直接运行

在启动目录的 `.env` 写入自己的配置；文件已被 Git 忽略。例如：

```dotenv
DEEPSEEK_API_KEY=your-key
```

然后启动：

```sh
cargo run
```

默认使用当前目录作为工作区。终端里可以连续提问，方向键编辑或召回输入；`/new` 开始新对话但保留文件，`/exit` 或 Ctrl-D 退出。Ctrl-C 可以取消当前回合；取消后先检查文件，再用 `/new` 开始。

```sh
cargo run -- -p "检查 README，简要说明这个项目"
cargo run -- --workspace path/to/project
printf '检查 README\n' | cargo run --quiet
```

`-p`（也可写为 `--print`）回答后退出，stdout 只保留回答；进度在 stderr。位置任务参数继续兼容。普通管道输入整体作为一次任务，`--chat` 则按行进行多轮对话。

程序只读取启动目录 `.env`，不会执行里面的 shell 内容；进程环境值优先。也支持 `OPENAI_API_KEY`、`OPENAI_BASE_URL`、`CRABLOOP_MODEL`，以及完整端点 `CRABLOOP_API_URL`。若同时配置多个 provider，可用 `CRABLOOP_PROVIDER=deepseek` 或 `openai` 明确选择。不要把真实密钥写入 README 或提交到 Git。

本版终端界面使用 rustyline，支持滚动阅读、输入编辑、工具进度和多轮历史。回答仍是非流式返回。更完整的终端布局留在后续主线。

## 逐步生长的里程碑

| 版本 | 核心实现目标 | 要亲手弄清楚的问题 |
|---|---:|---|
| v1 | 约 300 行 Rust，四个工具 | 模型如何根据真实工具反馈继续行动？ |
| v1.1（本分支） | 571 行 Rust，提前补上交互入口 | 多轮提问怎样复用同一份历史？ |
| v2 | 约 500 行 Rust，上下文管理与压缩 | 每轮模型看到什么，历史怎样被选择和整理？ |
| v3 | 约 1000 行 Rust，home、TOML、provider 与模型目录 | 怎样配置一次，持续使用和切换模型？ |
| v4 | Skills、web_fetch、web_search | 怎样复用方法，完成联网资料任务？ |
| v5 | JSONL 会话保存与恢复 | 中断之后如何接着做？ |

v3 优先完成配置和模型目录；有余量可提前体验最小 Skills，正式的 Skills 与联网实践安排在 v4。具体安排见[后续路线](docs/roadmap.md)。

模型接入使用 **OpenAI 原生 Responses API**。v1 提供四个工具：

- `read_file`：读取文件。
- `write_file`：创建或覆盖文件。
- `edit_file`：唯一匹配的精确文本替换。
- `bash`：在工作目录中执行 Bash 命令，收集结果。

v1 的验收是一次完整的修复过程：读取代码 → 测试失败 → 修改 → 再次测试 → 独立验证。

v2 增加上下文预算、历史整理和摘要压缩，通过实验检查任务约束、修改记录和测试结果是否得到保留。

## 学习与开发方式

教程会从一个空 Cargo 工程开始，用耐心、连贯的中文叙述陪你一步步写下代码。先看看当前程序能做什么，再从运行结果中发现下一个问题，让每个工具、循环和上下文机制都有出现的理由。

每一步都包含必要的原理、亲手实现、故障实验、验证和复盘。通过 Git 提交记录演进，在里程碑完成后保留 v1、v2 tag，方便回到相应版本学习。

从[教程入口](tutorial/README.md)了解这条路线。

行数计算覆盖全部 src/**/*.rs，测试、文档和练习仓库另算。代码规模是弹性的范围预算，v1 在 350–360 行也可以接受；保留正常格式化与必要错误处理。

## 当前进度与文档

原 v1 的教程 00–05 和[真实任务验收](evidence/v1/acceptance.md)保留不变；v1.1 增加[交互补篇](tutorial/v1/06-conversation.md)。本次回移通过 43 项 Rust 测试、3 项 Python 验收器回归、fmt 和 clippy；没有把旧 v1 的真实模型证据当作本次交互实测。

会话保存在进程内的 `Vec<Value>`，后续提问携带旧问题、工具记录和最终回答。请求 JSON 默认限制 131,072 字节（可用 `CRABLOOP_CONTEXT_BYTES` 调整），超过后明确停止；本快照不压缩、不落盘、不恢复跨进程历史。模型或协议错误后需要 `/new`，不会自动重复可能已有副作用的工具。固定工具边界与原 v1 相同：Bash 有超时和输出限制，但没有 OS sandbox，也不保证取消所有后代进程。

Cargo 工程放在仓库根目录；教程按阶段组织在 tutorial/v1/ 和 tutorial/v2/；实际开发记录放在 docs/devlog/。

- [设计说明](docs/design.md)：版本边界、协议、运行限制、上下文策略与完成标准。
- [完整实施与学习计划](docs/plan.md)：10 个学习站点、9 个实施任务、故障实验和验收方法。
- [教程写作约定](docs/writing-guide.md)：叙述方式、代码讲解、实验和章节质量要求。
- [仓库起步记录](docs/devlog/00-bootstrap.md)：初始化、目录与写作方式的决定。
- [开发约定](AGENTS.md)：项目目标、核心契约和验证要求。
