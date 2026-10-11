# CrabLoop：从一轮行动，长到自己的工作台

更新：2026-10-11。v1/v2 已按[实施计划](plan.md)完成；v3–v5 是既定后续规划，v6–v8 是新增建议，本文中的配置和命令尚未实现。

最值得保留的是逐步生长的节奏。先看见模型调用一次工具，再看见历史变长，最后才遇到“每次都要设置环境”“换模型不方便”“明天怎样接着做”。每版解决一个能亲手观察的问题。

## 逐步生长的版本主线

| 版本 | 范围预算 | 核心问题 | 内容与一个代表任务 |
|---|---:|---|---|
| v1 | 约 300 行；350–360 也可接受 | 模型怎样行动并依据反馈调整？ | Responses + read_file/write_file/edit_file/bash；修复函数，再做一次多文件资料整理 |
| v2 | 约 500 行，允许合理浮动 | 历史变长后，下一轮看到什么？ | 根 AGENTS.md、完整交互组、输入预算、摘要、JSONL 运行记录；小预算下完成任务 |
| v3 | 约 1000 行，优先保持可读 | 怎样成为不用反复配置的个人 CLI？ | `~/.crabloop`、TOML、凭证、provider 预置/Custom、模型目录与缓存；换 provider 执行同一任务 |
| v4 | 先按功能估算，不预设硬上限 | 怎样复用工作方法，并查阅外部资料？ | Skills 最小支持、web_fetch、web_search、CLI 配置菜单；查官方资料，保存带来源的比较报告 |
| v5 | 先按功能估算，不预设硬上限 | 进程退出后，怎样知道做到哪里并继续？ | JSONL sessions、list/show/resume、压缩状态重建、取消与未完成操作；中断后核对并继续 |
| v6（建议） | 分别记录核心与界面增量 | 怎样在终端持续协作、观察和打断？ | 流式 REPL、明确事件，再增加 Ratatui TUI；同一任务连续修改 |
| v7（建议） | 先做可解释的小型 memory | 新会话应该记住哪些旧经验？ | 分作用域记忆、来源、检索、修正和遗忘；跨会话复用验证过的项目知识 |
| v8（选修） | 按对照结果决定是否采用 | 哪些基础能力适合交给 Rust 库？ | 固定版本的 Rig 适配实验；保留手写 loop，比较协议行为与维护成本 |

行数用于提醒我们收住范围，统计全部 src/**/*.rs，测试、教程与开发脚本单独记录。不会为了从 360 降到 300 而压缩难读的代码，也不会因为模型多输出一个正确的统计字段就判失败。验收看任务事实、工具结果和程序的关键边界。

v3 若完成配置与目录后仍有余量，可以提前加入最小 Skills：只列名称/简介、按需读 SKILL.md。它是候选项，不阻塞 v3；完整讲解与联网实践归 v4。Streaming 与终端交互在 v6 单独展开；MCP、插件市场、多 agent 调度继续留作后续候选。

## v3：先给程序一个家

程序代码仍是仓库根目录的一个 Cargo 工程。用户数据放在自己的 home，默认布局如下；目录随对应版本出现，不提前创建空功能。

```text
~/.crabloop/
├── config.toml                 # v3：provider、默认模型、预算等普通设置
├── credentials.toml            # v3：本机凭证，单独保存
├── cache/
│   └── models-<profile>.json    # v3：可重建的模型目录
├── skills/<name>/SKILL.md       # v4：用户安装的工作方法
├── sessions/<session-id>.jsonl  # v5：跨进程会话
└── memory/                     # v7：筛选后的跨会话知识，按全局/项目分开
```

允许 `CRABLOOP_HOME` 指向另一个目录，便于测试与临时 profile。默认不需要 `.env`；保留进程环境覆盖，方便 CI。普通配置优先级为 CLI > 显式环境覆盖 > config.toml > 内置默认值。凭证按已选 provider 查找，不能因为一个通用 OPENAI_API_KEY 存在就把它发送给别的 provider。

计划中的 config.toml 示例：

```toml
version = 1
provider = "deepseek"
model = "deepseek-flash"

[context]
max_request_bytes = 49152

# 内置 provider 可省略 base_url；写出时表示明确覆盖。
[providers.deepseek]
base_url = "https://api.deepseek.com"

[providers.minimax]
base_url = "https://api.minimaxi.com/v1"

[providers.openai]
base_url = "https://api.openai.com/v1"

[providers.teamorouter]
base_url = "https://api.teamorouter.cn/v1"
models_path = "/models"
models = ["gpt-6-luna"]  # 手动补充；发现接口不可用时仍可选用
```

所有 provider 共用 Responses 适配器，`base_url` 是 API 前缀，后接 `/responses`。没有必要先引入多协议 trait。内置预置解决名称、端点与默认值，Custom 使用相同数据结构。MiniMax 国际站端点可由用户明确覆盖，凭证与所选站点匹配。

credentials.toml 按 provider/profile 保存 API Key；目录在 Unix 下使用 0700，凭证文件使用 0600。配置示例、目录缓存和会话不写密钥。最初允许手工编辑；随后加 `crabloop auth set deepseek`，从隐藏输入读取，避免把密钥放进 CLI 参数。这是本机文件存储，后续才考虑系统钥匙串。

建议拆成三个可运行增量：先加载 TOML 并沿用已有 Config；再加三个预置与 Custom；最后实现 `crabloop models --provider deepseek` 和 `--refresh`。v3 先完成文件配置与命令入口；v4 再补一个读写同一套文件的配置菜单，保持每一步清楚。自动选模型和自动切换 provider 另行评估。

## 模型目录：发现、补充、选择

截至本次核查，三个目标 provider 都有官方 Responses 与 OpenAI 风格模型目录：

| Provider | Responses | 模型目录 | 当前证据 |
|---|---|---|---|
| OpenAI | `/v1/responses` | `/v1/models` | 官方文档；本项目 gpt-6-luna 的真实验收走 TeamoRouter，不能算官方端点实测 |
| DeepSeek | `/responses` | `/models` | 官方文档 + 本项目真实目录/Responses/工具循环 |
| MiniMax 国内站 | `/v1/responses` | `/v1/models` | 官方文档；尚未用本项目凭证实测 |
| Custom | `<base>/responses` | 默认 `<base>/models`，允许配置路径或禁用 | 以用户端点返回和实际调用为准 |

因此只做 Responses 是合理的范围选择。但“OpenAI compatible”有时只代表 Chat Completions，不能自动判定支持 Responses。`/models` 的存在也不是工具调用能力证明。OpenAI 的目录提供基础属性，DeepSeek 则提供更多能力字段；缺失的能力记作 unknown，不猜测上下文长度或自动挑所谓最新模型。

目录流程保持简单：显式刷新拉取 `data[].id` 与可用元信息 → 去重并合并用户手动模型 → 保存缓存与更新时间 → 用户明确选择模型。默认使用上次目录，`--refresh` 主动更新；刷新失败保留旧目录并标明陈旧，无缓存则使用手动项。目录不阻塞已经配置的模型运行。有效空列表与网络失败分开处理。

缓存属于 provider profile，并记录端点与本地凭证修订标识；更换端点或凭证后重新发现，避免把另一个账号的可用列表当成当前结果。缓存不保存密钥。模型选择是 CLI/配置能力，无需新增一个让模型给自己换 provider 的工具。

参考实现读到的是固定本地提交：

- [Hermes models.py](https://github.com/NousResearch/hermes-agent/blob/e2f8a0731bf26e95b31e35d73e71e183a1045b81/hermes_cli/models.py)：`probe_api_models`、`cached_fetch_api_models` 展示真实 `/models` 探测和缓存；[model_catalog.py](https://github.com/NousResearch/hermes-agent/blob/e2f8a0731bf26e95b31e35d73e71e183a1045b81/hermes_cli/model_catalog.py) 另外处理维护者目录。CrabLoop 借鉴“目录来源可区分，失败保留已知值”。
- [Pi config.ts](https://github.com/earendil-works/pi/blob/216e672e7c9fc65682553394b74e483c0c9e47f7/packages/coding-agent/src/config.ts) 将 models.json、auth.json、settings.json 放在 `~/.pi/agent`；[models.ts](https://github.com/earendil-works/pi/blob/216e672e7c9fc65682553394b74e483c0c9e47f7/packages/ai/src/models.ts) 区分静态列表与动态刷新。该提交的 OpenAI/DeepSeek 工厂使用生成的静态目录，不能概括成“Pi 所有模型都实时拉取”。CrabLoop 借鉴配置、凭证、模型元信息分开管理。

官方协议依据：[OpenAI models](https://developers.openai.com/api/reference/resources/models/methods/list)、[DeepSeek Responses](https://api-docs.deepseek.com/api/create-response/)、[DeepSeek models](https://api-docs.deepseek.com/api/list-models/)、[MiniMax Responses](https://platform.minimaxi.com/docs/api-reference/responses-create)、[MiniMax models](https://platform.minimaxi.com/docs/api-reference/models/openai/list-models)。这些是 2026-10-11 的核查结果，实施 v3 时再复核字段和端点。

## v4：工具负责取资料，Skills 负责组织工作

web_fetch 与 web_search 值得做，它们让通用任务不再只围绕本地文件。但可以等行动循环、上下文和配置都清楚以后再引入。

先以简单 HTTP 抓取讲清 `web_fetch(url)` 的参数、超时、输出限额与错误回传，再接入 Firecrawl 作为正文抓取后端；默认只请求网页正文/Markdown，返回来源 URL 和截断信息。这个教学版本不开放浏览器交互、登录或 PDF 功能。

`web_search(query)` 首先接 Tavily，返回标题、URL、摘要和来源时间（有才返回）。初始组合为 Tavily Search + Firecrawl Scrape；模型看到的工具名始终是 web_search/web_fetch，后端由配置决定。Rust 沿用 reqwest 调用服务 HTTP API，先实现这两个具体适配，不引入通用插件框架，也不承诺两家服务的全部功能可以互换。

接口依据：[Tavily Search](https://docs.tavily.com/documentation/api-reference/endpoint/search)、[Firecrawl Scrape](https://docs.firecrawl.dev/features/scrape)。已核对公开文档，尚未验证本项目账号；用户提供的本地 .env 将在接入时用于对应服务。搜索/抓取 Key 与模型 Key 分开配置，服务端内置 web_search 留作之后的对照。

### 菜单与手工配置使用同一套文件

v4 增加计划中的 `crabloop config` 交互菜单，先做简单终端问答，选择配置项、输入/更换服务 Key、保存，并显示当前生效值的来源。该菜单读写的就是 ~/.crabloop 下的 TOML 文件，手动编辑后下一次启动同样生效；不另存一份菜单专用设置。

普通设置放在 `~/.crabloop/config.toml`，例如：

```toml
[web.search]
provider = "tavily"
max_results = 5

[web.fetch]
provider = "firecrawl"
max_output_bytes = 12000
```

Key 放在 `~/.crabloop/credentials.toml`，也可手工编辑：

```toml
[tavily]
api_key = "<your-tavily-key>"

[firecrawl]
api_key = "<your-firecrawl-key>"
```

菜单输入 Key 时隐藏输入，查看配置时只显示已配置状态。已有 `.env` 可通过菜单的“从指定 .env 导入”入口迁移所选的 TAVILY_API_KEY/FIRECRAWL_API_KEY；按数据解析，不执行文件，也不导入无关凭证。导入以后日常使用无需依赖该 .env。进程环境中的服务专用 Key 可以覆盖 credentials.toml，普通设置继续遵循 CLI > 显式环境覆盖 > config.toml > 默认值。

如果环境变量正在覆盖文件，菜单要明确显示来源，避免用户修改文件后误以为程序忽略配置。实现时验证菜单写入与手改文件得到同样的运行配置，再用现有账号验证一次搜索和一次抓取；目前这里只记录设计，配置功能尚未实现。

Skills 是另一层：例如“技术方案调研”说明先搜索官方资料、逐页读取、比较差异、记录不确定项，再用 write_file 输出报告。它复用 web_search/web_fetch/bash/read_file，不凭一段 SKILL.md 自动获得联网能力。

如果只为了完成某个任务，Skill 调用既有 CLI 当然可行；但这个项目要学习 harness，亲手实现两个小型网络工具更容易观察参数、失败、输出限额和上下文增长。建议采用“核心有稳定的网络工具，Skill 教模型怎样组合使用”。外部网页与 Skill 文档进入明确的数据/指令位置，网页内容不能修改全局配置或授权。

Skills 最小范围：从约定目录发现 SKILL.md 的 name/description，目录摘要加入模型可见信息，需要时再加载正文与明确引用的文件。先手工安装、只读加载；不做远程自动安装、插件运行时、OAuth 或 MCP。跨工作区 skill 路径用显式 loader 处理，不扩大 read_file 的现有路径契约。

代表任务：比较两个库的官方 API，生成含原始 URL 的 Markdown 表格。验证来源能对应结论，不要求固定措辞或固定调用顺序。

## v5：保存的是可解释的会话

JSONL 适合这个教学版本：每行一个事件，能追加、能逐行查看，也便于解释中断留下的痕迹。先做单进程、单写入者，不引入数据库和分支树。

文件头记录 schema_version、session_id、工作目录、provider/model 与开始时间；后续事件有递增 seq，记录用户任务、模型输出、tool_started、tool_finished、压缩提交与停止。凭证不写入记录。摘要是历史的一种投影，完整事件仍然保留。

实现顺序：`sessions list/show` → 重建已完成组及最近一次有效摘要 → `resume <id>` 接收新任务 → 处理 Ctrl-C 与尾部半行。恢复时遇到 tool_started 而没有 tool_finished，只能认定结果未知：先检查文件或外部状态，再决定后续动作，不能直接重放写入与命令。

v2 的 `.crabloop/run-*.jsonl` 只用于观察一次运行，已经可以教“完整记录与模型输入不同”。v5 才赋予日志稳定 session 身份、版本和恢复语义，并移到 home 下；早期日志不承诺自动迁移为可恢复会话。

## v6 建议：先接好交互，再搭终端界面

沿用之前提出的流式 REPL 路线，先让同一个 session 连续接收任务，逐步显示模型文字与工具状态，并把 Ctrl-C 变成有明确结果的回合取消。随后再添加界面布局。v5 已定义的中断/恢复语义继续复用，不在界面层重新实现一套。

终端库按实际问题选择：

| 时点与用途 | 建议组件 | CrabLoop 自己负责什么 |
|---|---|---|
| v4 的 config 菜单 | dialoguer：选择、输入、隐藏密码输入 | 配置校验、优先级、存储与生效来源 |
| v6 的完整 TUI | Ratatui：布局、段落、列表、滚动与状态栏 | 会话视图、焦点、事件处理与状态更新 |
| 终端输入和终端模式 | Crossterm | 输入事件怎样变成提交、取消或选择动作 |
| 多行输入框 | 选与 Ratatui 版本兼容的文本编辑组件；如维护中的 tui-textarea-2 候选 | 提交快捷键、粘贴体验、与当前任务的关系 |

当前官方资料提供这些基础能力：[dialoguer](https://docs.rs/dialoguer/latest/dialoguer/)、[Ratatui](https://ratatui.rs/)、[Crossterm](https://docs.rs/crossterm/latest/crossterm/)。文本输入组件的维护状态与依赖版本在实施时复核；这里不固定尚未构建验证的组合。

核心通过 channel 发出 TextDelta、ToolStarted、ToolFinished、Compaction、Stopped 等事件，CLI/TUI 消费事件并展示。界面刷新不执行工具，模型网络等待不阻塞按键。当前单次 CLI 入口仍可用于脚本和验收；TUI 复用同一个运行核心。可以在具体需要出现时拆出 src/ui/，所有生产 Rust 仍计入总行数，另列核心/界面分项帮助阅读。

最小界面只有对话区、输入区和状态栏，工具详情可展开。先验证中文粘贴、多行输入、窗口缩放、流式输出与取消，以及退出后的终端恢复。Markdown 美化、主题与复杂快捷键随后按需要添加，不把一个版本变成终端编辑器项目。

## v7 建议：记住可复用的经验

v2 的上下文压缩负责这一轮看什么；v5 的 session 记录负责同一段工作如何恢复。v7 的 memory 从历史中筛选将来仍有用的信息，让新 session 可以复用。AGENTS.md 继续作为显式项目约定，memory 提供带来源的参考事实，不覆盖当前任务与项目规则。

例如，一个项目实际使用的验证命令，或用户明确表达的报告偏好，可能值得保留；一次完整终端输出通常留在 session 即可。先从少量明确记忆开始，让每次写入和召回都能解释。

建议的三个增量：

1. **可查看的存储与显式写入。** 用人可读 Markdown 条目和少量元数据保存内容、作用域、来源 session/事件位置、更新时间。先支持用户明确要求“记住”，以及 memory list/show/add/forget；文件放在 ~/.crabloop/memory 下，项目知识按 workspace 标识隔离，全局偏好单独存储。
2. **有预算的检索。** 从项目范围和关键词检索起步，只把相关条目放进当前上下文，记录召回的 ID 与来源。检索文本计入 v2 的请求预算，不在每轮全量注入所有记忆。
3. **修正与提炼。** 新事实可以更新或替代旧事实，用户能删除或停用条目；对易变化的信息保留时间并在需要时重查。自动提炼先产出候选，后续再评估何时允许自动保存；未经核对的模型推断不直接变成确定事实。

memory 不保存密钥，也不把工具/网页里的指令提升成全局规则。删除记忆后，后续检索不能再返回它；原 session 日志有自己的生命周期，界面需把两者的删除含义讲清楚。

第一版检索用本地文件与简单匹配即可。只有出现“知道意思却搜不到关键词”的具体案例，才比较 SQLite FTS、embedding 或向量存储；检索命中并不证明内容仍然正确。Rig 可以在后续实验里提供 embedding/存储接口，但不能替 CrabLoop 决定该记什么、保留多久或冲突时信谁。

代表练习：记住项目验证命令 → 结束 session → 新会话召回并使用 → 用户更正命令 → 新会话使用更新值 → 删除后不再召回；另一个项目不收到这条项目记忆。这比先接向量数据库更容易看清 memory 的行为。

## v8 选修：把手写实现与 Rig 放在一起比较

Rig 适合列入高阶实验。它的当前官方文档覆盖模型/provider、工具、流式、embedding、向量存储与 memory，并明确允许调用模型、保留自有 loop；也提供更完整的 agent runtime。[Rig 官方介绍](https://rig.rs/docs)、[当前 API 入口](https://docs.rs/rig/latest/rig/)

建议先在独立的实验开发分支里接一个 Rig 模型后端，保留 CrabLoop 的循环、上下文策略、工具执行与 session/memory 规则。用相同 fixture 和代表任务比较原手写 HTTP 与 Rig 适配：调用/结果匹配、reasoning 与 phase 回传、流式中断、usage/错误、工具副作用和预算是否仍有相同含义。尤其不能因为 API 都叫 Responses 就假定原生字段一定完整往返。

实验需要固定实际采用的已发布版本，阅读对应源码/API，而不是直接把 main 分支文档当稳定契约。记录依赖变化、适配代码和调试成本，解释哪些重复代码减少了、哪些状态仍由 harness 持有。通过对照后，再决定并入后续里程碑还是作为选修保留。使用 Rig agent runtime 则是第二个独立对照题，不与替换模型客户端同时发生。

Rig 的当前 API 也有图像生成接口。如果之后希望 CrabLoop 增加生图任务，可以单独规划 generate_image 工具与产物文件管理；它与引入通用 agent 基础库分开决定，不影响当前 Responses 主循环路线。

这些建议的目的，是在已经理解 harness 状态之后，学习如何选择和约束第三方抽象。终端布局与输入组件可以较早复用；模型执行与记忆策略先保持可观察，再用实验决定复用边界。

## 教程与提交继续沿用同一条路

仓库继续使用 `tutorial/vN/` 组织教学，`docs/devlog/` 留下真实经过，`docs/` 放设计、路线与证据解释。每版新增文件时再建相应目录。每个可验证增量提交并推送。每个完成版本保留 `milestone/vN` 学习分支及不移动的 `vN` 标签，v6/v7 也沿用；后续开发另开分支，学习分支保持稳定。阶段文档明确哪些已实现，哪些仍是计划。
