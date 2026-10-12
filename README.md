# CrabLoop 🦀

A small coding-agent harness that grows step by step in Rust.

用 Rust 从零实现一个小型 coding agent，通过读取代码、修改文件、运行测试和处理反馈，练习 agent harness 的核心机制。

![CrabLoop 两轮真实交互：创建文件，再接着修改](docs/assets/crabloop-chat.gif)

真实 DeepSeek V4.1 Flash 录制：第一轮创建并读取文件，第二轮沿用对话修改同一个文件。GIF 缩短等待间隔；[录制与独立验证](evidence/interactive/README.md)。

项目采用逐步开发的方式：先用约 300 行写出最小循环，再加入终端交互与上下文管理，随后规划约 1000 行的 v3。通过 coding 和通用本地任务，逐步练习同一套 harness。

## 从哪里开始

第一次体验建议使用 **main**，在仓库根目录配置 `.env` 后直接 `cargo run`。按学习目的选择其他快照：

| 入口 | 适合学习什么 | 启动与交互 |
|---|---|---|
| `main` | 当前完整实现：四工具、多轮会话和上下文压缩 | `cargo run` 聊天；`cargo run -- -p "任务"` 单次执行 |
| `milestone/v1` | 四工具循环、默认工作目录、.env、输入编辑和多轮 Session | 与 main 相同的聊天和 `-p` 入口；完整源码 571 行，无压缩或 journal |
| `milestone/v2` / `v2` | 504 行的上下文投影与摘要压缩 | 保留当时的显式参数入口；便捷交互已接入 main |

想学习带交互的第一版，可以运行：

```bash
git switch milestone/v1
cargo run
```

阅读顺序是 [00–05 最小循环](tutorial/README.md) → [05b 多轮交互](tutorial/v1/05b-interactive.md) → [06–09 上下文管理](tutorial/v2/06-context-projection.md)。v1 学习分支已经包含使用优化；原始 314 行闭环保留在 `step-05-v1` / `v1` 历史标签。进入 v2 时按章节选择对应代码基线，main 同时具备交互和压缩。完整关系见 [版本与 Git 工作流](docs/git-workflow.md)。

## 逐步生长的里程碑

| 版本 | 核心实现目标 | 要亲手弄清楚的问题 |
|---|---:|---|
| v1 | 原始闭环 314 行；含 CLI/UI 的当前分支 571 行 | 模型如何行动，并沿着同一份历史继续提问？ |
| v2 | 约 500 行 Rust，上下文管理与压缩 | 每轮模型看到什么，历史怎样被选择和整理？ |
| v3 | 约 1000 行 Rust，home、TOML、provider 与模型目录 | 怎样配置一次，持续使用和切换模型？ |
| v4 | Skills、web_fetch、web_search | 怎样复用方法，完成联网资料任务？ |
| v5 | JSONL 会话保存与恢复 | 中断之后如何接着做？ |
| v6（建议） | 流式输出与完整 Ratatui TUI | 怎样让长任务、工具详情与终端布局更易观察？ |
| v7（建议） | 跨会话 memory | 哪些经验值得保存、召回、修正与遗忘？ |
| v8（选修） | Rig 后端对照 | 哪些基础能力适合交给第三方 Rust 库？ |

基础多轮终端界面已提前实现，默认启动即可聊天，`-p` 执行一次任务。v3 优先完成配置和模型目录；正式的 Skills 与联网实践安排在 v4。具体安排见[后续路线](docs/roadmap.md)。

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

从[教程入口](tutorial/README.md)了解这条路线；[完整代码增量](tutorial/v1/patches/README.md)可从空目录逐步构建 v1，最后展开为 352 行的教学排版。再读[接着问下一句](tutorial/v1/05b-interactive.md)，体验 v1 分支的交互入口：571 代码行 / 581 物理行，含完整 CLI/UI，保留内存历史，尚无压缩或 journal。

行数计算覆盖全部 src/**/*.rs，测试、文档和练习仓库另算。代码规模是弹性的范围预算，v1 在 350–360 行也可以接受；保留正常格式化与必要错误处理。

## 当前进度与文档

当前已实现四工具、Responses 行动循环、上下文压缩和轻量多轮终端界面，教程 00–09 已完成。DeepSeek V4.1 Flash 与备选 gpt-6-luna 已通过真实修复任务；通用本地报告任务也有独立验收。固定 v1 为 314 行、v2 为 504 行；加入 CLI、交互和正常教学排版后的当前实现为 **813 代码行 / 825 物理行**，包含全部 src 文件，不再把它称作 300 行版本。[v1 验收](evidence/v1/acceptance.md) 和 [v2 压缩验收](evidence/v2/acceptance.md) 已保存。v3–v5 仍是规划，v6–v8 为建议与选修。

Cargo 工程放在仓库根目录；教程按阶段组织在 tutorial/v1/ 和 tutorial/v2/；实际开发记录放在 docs/devlog/。

- [设计说明](docs/design.md)：版本边界、协议、运行限制、上下文策略与完成标准。
- [完整实施与学习计划](docs/plan.md)：10 个学习站点、9 个实施任务、故障实验和验收方法。
- [教程写作约定](docs/writing-guide.md)：叙述方式、代码讲解、实验和章节质量要求。
- [仓库起步记录](docs/devlog/00-bootstrap.md)：初始化、目录与写作方式的决定。
- [开发约定](AGENTS.md)：项目目标、核心契约和验证要求。

## 运行与版本切换

需要 Rust 1.94.0 和 Bash。在仓库根目录直接启动：

```bash
cargo run
```

程序默认使用当前目录，进入滚动终端界面。直接输入任务，回答结束后可以继续追问；同一个会话保留前面的提问、回答和工具结果。方向键浏览本次进程中的输入历史；空行不提交，`/new` 开新会话，`/exit` 或 Ctrl-D 退出。Ctrl-C 在输入时清空草稿，在执行时取消当前回合。

当前目录的 `.env` 会自动作为配置数据读取，已有进程环境优先。推荐只配置 `DEEPSEEK_API_KEY`，默认使用 `deepseek-flash`。仓库已有 `.env` 时直接使用；新的配置格式见 [.env.example](.env.example)。

单次任务用 `-p`（或 `--print`），回答后退出，stdout 只包含最终回答：

```bash
cargo run -- -p "检查 README，说明这个项目能做什么"
cargo run -- --workspace /path/to/project
cargo run -- --workspace /path/to/project -p "修复测试"
printf '%s\n' "检查当前目录" | cargo run
cargo run -- --help
```

自动选择遵循已有配置：没有显式模型/完整端点或进程级 OpenAI 配置时，优先使用 DEEPSEEK_API_KEY；否则沿用 OPENAI_API_KEY、OPENAI_BASE_URL、CRABLOOP_MODEL。可用 CRABLOOP_PROVIDER=deepseek/openai 明确指定。openai 模式默认模型为 gpt-6-luna，官方 DeepSeek base URL 则默认 deepseek-flash；其他服务应显式填写支持的模型名。CRABLOOP_API_URL 覆盖所选 provider 的完整端点，Key 始终来自该 provider 对应变量。

旧的任务位置参数仍可用。无参数管道输入作为一个完整任务；`--chat` 显式按行进行多轮对话。只读取启动目录的 .env，不向父目录搜索，也不执行文件里的命令。空会话退出不会创建日志或请求模型。当前界面使用 rustyline 编辑输入，模型回答仍整段显示；流式显示和完整屏幕布局留给 v6。

单次模式的 stderr 保留工具 JSON 轨迹和日志路径；交互模式显示工具进度。每个实际开始的会话写入工作区 .crabloop 的 JSONL，用于观察，不支持跨进程恢复。请求错误或中断后要先检查文件状态，再用 `/new`；这不会回滚已发生的修改。CRABLOOP_CONTEXT_BYTES 默认 49152，最小 4096。摘要请求要求模型支持 reasoning.effort=none；v2 主验收模型为 DeepSeek V4.1 Flash。

学习分支展示该版当前实现，标签保存固定历史快照；想动手修改时从标签另开自己的分支，例如 `git switch -c my-v1-exercise milestone/v1`。main 保持最新可运行实现，每个通过验证的小迭代都合入并推送；已完成的临时开发分支随后清理，教学提交与里程碑继续保留。
