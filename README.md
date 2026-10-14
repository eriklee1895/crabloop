<p align="center">
  <img src="docs/assets/crabloop-banner.png" alt="CrabLoop — A small agent harness, built step by step in Rust." width="640">
</p>

[![CI](https://github.com/eriklee1895/crabloop/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eriklee1895/crabloop/actions/workflows/ci.yml) [![Rust](https://img.shields.io/badge/built_with-Rust-d66b45?logo=rust&logoColor=white)](Cargo.toml) [![v1 under 400 lines](https://img.shields.io/badge/v1-%3C400_lines-299d91)](https://github.com/eriklee1895/crabloop/tree/milestone/v1) [![中文教程](https://img.shields.io/badge/教程-中文-299d91)](tutorial/README.md)

用 Rust 从零实现一个小型 coding agent，通过读取代码、修改文件、运行测试和处理反馈，练习 agent harness 的核心机制。

[开始学习](tutorial/README.md) · [快速运行](#运行与版本切换) · [路线图](docs/roadmap.md) · [需求 Todo](docs/todo.md)

![CrabLoop 两轮真实交互：创建文件，再接着修改](docs/assets/crabloop-chat.gif)

真实 DeepSeek V4.1 Flash 录制：第一轮创建并读取文件，第二轮沿用对话修改同一个文件。GIF 缩短等待间隔；[录制与独立验证](evidence/interactive/README.md)。

项目首先帮助学习者快速读懂一个 agent 核心循环：v1 目标约 300 行，全部生产 Rust 严格少于 400 行。复杂 Harness 在后续版本逐步生长。当前 v1 分支已收敛到 389 代码行 / 450 物理行，具体约束见[教学边界](docs/v1-scope.md)。

## 从哪里开始

学习 v1 统一从 **milestone/v1** 开始，在仓库根目录配置 `.env` 后运行一个任务。v1 不接受任务或帮助参数；README 与 .env.example 负责配置说明。后续阶段各有自己的入口：

| 入口 | 适合学习什么 | 启动与交互 |
|---|---|---|
| `milestone/v1` | v1 唯一学习代码：326 行 Harness 与入口 + 62 行 CLI | cwd 工作区；无参数启动，从 stdin 输入一次任务 |
| `milestone/v2` | 504 行的上下文投影与摘要压缩 | 保留当时的显式参数入口 |
| `main` | 当前完整实现：四工具、多轮会话和上下文压缩 | `cargo run` 聊天；`cargo run -- -p "任务"` 单次执行 |

想快速看懂最小循环，可以运行：

```bash
git switch milestone/v1
cargo run
```

阅读顺序是 [00–05 最小循环](tutorial/README.md) → [06–09 上下文管理](tutorial/v2/06-context-projection.md)；[多轮交互](tutorial/v1/05b-interactive.md)作为后续选读。直接阅读和从空目录逐章重建，都以同一份 `milestone/v1` 代码收束。进入 v2 时按章节选择对应代码基线，main 同时具备交互和压缩。完整关系见 [版本与 Git 工作流](docs/git-workflow.md)。

## 逐步生长的里程碑

| 版本 | 核心实现目标 | 要亲手弄清楚的问题 |
|---|---:|---|
| v1 | 目标约 300 行，硬上限 <400；当前 389 行 | 如何快速看懂模型—工具—反馈—停止的核心循环？ |
| v2 | 约 500 行 Rust，上下文管理与压缩 | 每轮模型看到什么，历史怎样被选择和整理？ |
| v3 | 约 1000 行 Rust，基础 SSE、home、TOML、provider、模型目录与轻量 context 信息 | 怎样配置一次、切换模型，并看清上下文用量？ |
| v3 后：可观测性专题 | 配置开关、observer、Rust OpenTelemetry → Langfuse v4 | 每个 step 看到了什么，工具反馈与压缩怎样影响下一轮？ |
| v4 | Skills、联网工具、todo_write、配置菜单 | 怎样复用方法、查资料，并跟踪长任务进度？ |
| v5 | JSONL 恢复、Todo 恢复、ask_user_question | 怎样恢复进度，并等待用户回答后继续？ |
| v6（建议） | 输出渲染 → slash command 交互 → 完整布局 | 怎样看清 step、工具反馈与停止，并通过键盘控制输入？ |
| v7（建议） | 跨会话 memory | 哪些经验值得保存、召回、修正与遗忘？ |
| v8（选修，可跳过） | Rig 后端对照，是否采用由实验决定 | 哪些具体重复工作适合交给第三方 Rust 库？ |

基础多轮终端界面已提前实现，默认启动即可聊天，`-p` 执行一次任务。v3 先学习基础 SSE，再完成个人配置和模型目录；随后单独验收可观测性专题，正式的 Skills 与联网实践安排在 v4。现有主版本编号保持；具体安排见[后续路线](docs/roadmap.md)，已排期与待排期需求统一见[需求 Todo](docs/todo.md)。

可观测性专题计划通过显式 telemetry 开关，把模型、工具、压缩和停止事件关联到 Langfuse。遥测默认关闭，本地 JSONL 保留执行记录。v6 先区分 You/CrabLoop、Progress、可读推理与工具结果，再增加 Markdown、详情展开、slash 候选菜单和完整布局；step 的含义和计数边界会在教程中解释。观测与丰富 TUI 尚待实现，Langfuse 账号和实际入库尚未验证。Rig 保持选修，v1–v7 主线可独立完成。

后续常用工具按状态机制逐步加入：v4 用 todo_write 维护当前任务的计划；v5 恢复 Todo，并用 ask_user_question 保存待答问题、接收回答和继续原任务；v6 完善列表与问答界面。step 显示循环轮次，Todo 显示计划进度，完成项仍需实际验证。llm-space 的工具契约与交互作为学习参考，固定源码入口与验收边界见[路线图](docs/roadmap.md)。这些工具尚未实现，v1–v3 保持最小四工具范围。

模型接入使用 **OpenAI 原生 Responses API**。v1 提供四个工具：

- `read_file`：读取文件。
- `write_file`：创建或覆盖文件。
- `edit_file`：唯一匹配的精确文本替换。
- `bash`：在工作目录中执行 Bash 命令，收集结果。

v1 的验收是一次完整的修复过程：读取代码 → 测试失败 → 修改 → 再次测试 → 独立验证。

v2 增加上下文预算、历史整理和摘要压缩，通过实验检查任务约束、修改记录和测试结果是否得到保留。

## 学习与开发方式

教程会从一个空 Cargo 工程开始，用耐心、连贯的中文叙述陪你一步步写下代码。先看看当前程序能做什么，再从运行结果中发现下一个问题，让每个工具、循环和上下文机制都有出现的理由。

每一步都包含必要的原理、亲手实现、故障实验、验证和复盘。通过 Git 提交记录演进；每个阶段只有一个 milestone 学习分支，历史标签保存固定步骤与验收来源。

从[教程入口](tutorial/README.md)了解这条路线；[完整代码增量](tutorial/v1/patches/README.md)从空目录逐步构建到 `milestone/v1`，最终全部 src 文件与该分支一致。v1 共 389 代码行，其中 CLI 为 63 行，其他实现合计 326 行；完成核心学习后可选读[接着问下一句](tutorial/v1/05b-interactive.md)，该交互专题使用历史提交，日常界面保留在 main。

行数计算覆盖全部 src/**/*.rs，测试、文档和练习仓库另算。v1 的 <400 行是硬约束，其他版本按范围预算控制；保留正常格式化与必要错误处理，超预算时减少功能。

## 当前进度与文档

当前已实现四工具、Responses 行动循环、上下文压缩和轻量多轮终端界面，教程 00–09 已完成。DeepSeek V4.1 Flash 与备选 gpt-6-luna 已通过真实修复任务；通用本地报告任务也有独立验收。v1 学习代码为 397 行、v2 为 504 行；加入 CLI、交互和正常教学排版后的当前实现为 **853 代码行 / 869 物理行**，包含全部 src 文件，不再把它称作 300 行版本。[v1 GA 验收](evidence/v1/ga-review.md) 和 [v2 压缩验收](evidence/v2/acceptance.md) 已保存。v3–v5 仍是规划，v6–v8 为建议与选修。

Cargo 工程放在仓库根目录；教程按阶段组织在 tutorial/v1/ 和 tutorial/v2/；实际开发记录放在 docs/devlog/。

- [设计说明](docs/design.md)：版本边界、协议、运行限制、上下文策略与完成标准。
- [完整实施与学习计划](docs/plan.md)：10 个学习站点、9 个实施任务、故障实验和验收方法。
- [v1 GA 审查](evidence/v1/ga-review.md)：定稿提交与验收、格式与文件组织、离线检查和真实任务证据。
- [调试指南](docs/debugging.md)：main 的单步调试、JSONL 对照和 Proxyman HTTP 观察。
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

main 的 provider 选择按下表从上到下匹配：

| 条件 | 选择 | 凭证变量 |
|---|---|---|
| 显式配置 CRABLOOP_PROVIDER | 使用指定的 deepseek 或 openai | 对应 provider 的 Key |
| 未指定 provider，有 DEEPSEEK_API_KEY，且没有下述显式通用配置 | deepseek | DEEPSEEK_API_KEY |
| 其余情况 | openai | OPENAI_API_KEY |

“显式通用配置”指 CRABLOOP_MODEL 或 CRABLOOP_API_URL 已配置（进程环境或 .env），或进程环境中存在非空的 OPENAI_API_KEY / OPENAI_BASE_URL。openai 模式中的 DeepSeek hostname 判断只影响默认模型，不改变 provider 或 Key 来源。需要固定选择时，直接设置 CRABLOOP_PROVIDER。

旧的任务位置参数仍可用。无参数管道输入作为一个完整任务；`--chat` 显式按行进行多轮对话。只读取启动目录的 .env，不向父目录搜索，也不执行文件里的命令。空会话退出不会创建日志或请求模型。当前界面使用 rustyline 编辑输入；每轮完整响应返回后，commentary 以 `Progress` 段落显示在本轮工具之前，最终答案单独显示；基础流式显示安排在 v3，完整屏幕布局留给 v6。

单次模式的 stderr 保留过程说明、工具 JSON 轨迹和日志路径；交互模式显示工具进度。每个实际开始的会话写入工作区 .crabloop 的 JSONL，用于观察，不支持跨进程恢复。请求错误或中断后要先检查文件状态，再用 `/new`；这不会回滚已发生的修改。CRABLOOP_CONTEXT_BYTES 默认 4194304（4 MiB），最小 4096。main 的 max_output_tokens 为 32,768，摘要正文上限为 16 KiB；当前 milestone/v1 请求预算同为 4 MiB，max_output_tokens 为 131,072（128K）。请求字节预算不等于模型的 token 窗口，历史 v1/v2 标签保留原默认值。摘要请求要求模型支持 reasoning.effort=none；v2 主验收模型为 DeepSeek V4.1 Flash。

学习分支展示该版当前实现，标签保存固定历史快照；想动手修改时从学习分支另开自己的分支，例如 `git switch -c my-v1-exercise milestone/v1`。main 保持最新可运行实现，每个通过验证的小迭代都合入并推送；已完成的临时开发分支随后清理，教学提交与里程碑继续保留。

专业 CLI Help 将作为后续独立增量，设计 Usage、分组、对齐、高亮与多色，并兼顾无颜色输出和无需凭证查看。[设计边界](docs/cli-help-plan.md)；main 的聊天与 -p 单次执行入口继续保留。
