# CrabLoop 🦀

A small coding-agent harness that grows step by step in Rust.

用 Rust 从零实现一个小型 coding agent，通过读取代码、修改文件、运行测试和处理反馈，练习 agent harness 的核心机制。

项目采用逐步开发的方式：先完成约 300 行的 v1，再扩展到约 500 行的 v2，随后规划约 1000 行的 v3。通过 coding 和通用本地任务，逐步练习同一套 harness。

## 逐步生长的里程碑

| 版本 | 核心实现目标 | 要亲手弄清楚的问题 |
|---|---:|---|
| v1 | 约 300 行 Rust，四个工具 | 模型如何根据真实工具反馈继续行动？ |
| v2 | 约 500 行 Rust，上下文管理与压缩 | 每轮模型看到什么，历史怎样被选择和整理？ |
| v3 | 约 1000 行 Rust，home、TOML、provider 与模型目录 | 怎样配置一次，持续使用和切换模型？ |
| v4 | Skills、web_fetch、web_search | 怎样复用方法，完成联网资料任务？ |
| v5 | JSONL 会话保存与恢复 | 中断之后如何接着做？ |
| v6（建议） | 流式 REPL 与 TUI | 怎样连续协作、观察执行与取消？ |
| v7（建议） | 跨会话 memory | 哪些经验值得保存、召回、修正与遗忘？ |
| v8（选修） | Rig 后端对照 | 哪些基础能力适合交给第三方 Rust 库？ |

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

从[教程入口](tutorial/README.md)了解这条路线；[完整代码增量](tutorial/v1/patches/README.md)可从空目录逐步构建 v1，最后展开为 352 行的教学排版。

行数计算覆盖全部 src/**/*.rs，测试、文档和练习仓库另算。代码规模是弹性的范围预算，v1 在 350–360 行也可以接受；保留正常格式化与必要错误处理。

## 当前进度与文档

当前已实现四工具与 Responses 行动循环，教程 00–09 已完成。DeepSeek V4.1 Flash 与备选 gpt-6-luna 已通过真实修复任务；通用本地报告任务也有独立验收。固定 v1 为 314 行、v2 为 504 行；main 加入便捷启动后为 628 行。[v1 验收](evidence/v1/acceptance.md) 和 [v2 压缩验收](evidence/v2/acceptance.md) 已保存。DeepSeek 在 10 KB 预算下五次压缩后完成修复，独立测试通过。v3–v5 为路线规划，v6–v8 为新增建议与选修。

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

程序默认使用当前目录，显示当前 provider/模型，然后提示输入任务。空行可以重新输入，Ctrl-D 退出。当前入口执行一个任务后返回终端；连续会话留给后续交互版本。

当前目录的 `.env` 会自动作为配置数据读取，已有进程环境优先。推荐只配置 `DEEPSEEK_API_KEY`，默认使用 `deepseek-flash`。仓库已有 `.env` 时直接使用；新的配置格式见 [.env.example](.env.example)。

也可以直接给任务，或为脚本保留显式工作目录：

```bash
cargo run -- "检查 README，说明这个项目能做什么"
cargo run -- --workspace /path/to/project "修复测试"
printf '%s\n' "检查当前目录" | cargo run
cargo run -- --help
```

自动选择遵循已有配置：没有显式模型/完整端点或进程级 OpenAI 配置时，优先使用 DEEPSEEK_API_KEY；否则沿用 OPENAI_API_KEY、OPENAI_BASE_URL、CRABLOOP_MODEL。可用 CRABLOOP_PROVIDER=deepseek/openai 明确指定。openai 模式默认模型为 gpt-6-luna，官方 DeepSeek base URL 则默认 deepseek-flash；其他服务应显式填写支持的模型名。CRABLOOP_API_URL 覆盖所选 provider 的完整端点，Key 始终来自该 provider 对应变量。

只读取启动目录的 .env，不向父目录搜索，也不执行文件里的命令。模型配置加载后，交互输入或管道输入作为任务；没有输入时退出，不创建运行日志或发模型请求。

stderr 给出工具轨迹与工作区 .crabloop 内的 JSONL 路径。CRABLOOP_CONTEXT_BYTES 默认 49152，最小 4096。摘要请求要求模型支持 reasoning.effort=none；v2 主验收模型为 DeepSeek V4.1 Flash。固定 v1/v2 标签仍保留原有显式参数入口，以上便捷启动属于 main 的后续增量。

每版保留稳定学习分支与固定标签：`milestone/v1` + `v1`，`milestone/v2` + `v2`。使用 `git switch milestone/v1` 回到第一版，或 `git switch --detach v1` 精确查看标签；想动手修改时从标签另开自己的分支，例如 `git switch -c my-v1-exercise v1`。main 的文件保持最新可运行实现，并保留每轮提交历史；每个通过验证的小迭代都合入并推送 main。旧版本通过学习分支和标签查看，根目录保持一个 Cargo 工程。详细约定见 [Git 工作流](docs/git-workflow.md)。
