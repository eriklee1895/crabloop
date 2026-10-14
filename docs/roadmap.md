# CrabLoop：从一轮行动，长到自己的工作台

更新：2026-10-14。用户重新明确 v1 的 <400 行硬约束；当前 v1 已定稿为 397 行，历史快照保留。[教学边界](v1-scope.md)优先于此前的功能扩充安排。main 提前实现默认多轮终端聊天、-p 单次任务、当前目录 .env 和工具进度；milestone/v1 已收回单任务核心。后续安排包括 v3 后的可观测性专题、v4 的 todo_write、v5 的 ask_user_question 与恢复、v6 分步 TUI，以及 Rig 选修对照。下文这些能力仍待实现；Langfuse 账号连通性与实际 trace 尚未验证。

最值得保留的是逐步生长的节奏。先看见模型调用一次工具，再看见历史变长，最后才遇到“每次都要设置环境”“换模型不方便”“明天怎样接着做”。每版解决一个能亲手观察的问题。

2026-10-14 确认：v2–v3 继续以有限代码学习 Agent 核心机制，优先精简与可读性。自动重试、指数退避、复杂恢复和穷尽边缘情况的防御代码不作为这两版的主线或完成门槛。保留让所学机制成立的基本错误反馈、协议关联和明确停止；可靠性增强在以后有具体学习目标时作为独立主题展开。上下文预算继续使用简单、可复现的字节估算，不为计量精度引入 tokenizer。

## 逐步生长的版本主线

| 版本 | 范围预算 | 核心问题 | 内容与一个代表任务 |
|---|---:|---|---|
| v1 | 目标约 300 行，硬上限 <400；当前 397 行 | 怎样快速看懂一个完整 agent loop？ | 非流式 Responses、四工具、结果反馈、明确停止与必要限制；完成真实工具反馈闭环 |
| v2 | 约 500 行，允许合理浮动 | 历史变长后，下一轮看到什么？ | 根 AGENTS.md、完整交互组、输入预算、摘要、JSONL 运行记录；小预算下完成任务 |
| v3 | 约 1000 行，优先保持可读 | 怎样成为不用反复配置的个人 CLI？ | 基础 SSE、`~/.crabloop`、TOML、凭证、provider 预置/Custom、模型目录与缓存；分别学习流式显示、模型配置与轻量 context 信息 |
| v3 后：可观测性专题 | 单独估算增量，仍计入源码总量 | 每一步看到了什么，执行了什么，为什么继续或停止？ | 可选 telemetry、生命周期 observer、Rust OpenTelemetry → Langfuse v4；把终端轮次与模型、工具、压缩轨迹对应起来 |
| v4 | 先按功能估算，不预设硬上限 | 怎样复用方法、查资料，并跟踪长任务进度？ | Skills、web_fetch、web_search、todo_write、CLI 配置菜单；维护计划，保存带来源的比较报告 |
| v5 | 先按功能估算，不预设硬上限 | 怎样恢复进度，并在需要用户选择时等待和继续？ | JSONL sessions、list/show/resume、Todo 恢复、ask_user_question、取消与未完成操作；恢复已知状态和待答问题 |
| v6（建议） | 分别记录核心与界面增量 | 怎样观察长任务和丰富输出，并控制输入？ | 输出渲染 → slash command 交互 → 完整布局；保留可见的 step、预算、工具结果与停止原因 |
| v7（建议） | 先做可解释的小型 memory | 新会话应该记住哪些旧经验？ | 分作用域记忆、来源、检索、修正和遗忘；跨会话复用验证过的项目知识 |
| v8（选修，可跳过） | 按对照结果决定是否采用 | 哪些基础能力适合交给 Rust 库？ | 固定版本的 Rig 适配实验；主线独立推进，是否采用由协议验证与维护收益决定 |

v1 的 <400 行是硬边界：统计全部 src/**/*.rs，测试、教程与开发脚本单独记录。超预算时删减功能，不放宽 v1 的范围；其他阶段仍按范围预算控制。不会为了从 360 降到 300 而压缩难读的代码，也不会因为模型多输出一个正确的统计字段就判失败。验收看任务事实、工具结果和程序的关键边界。

v3 若完成配置与目录后仍有余量，可以提前加入最小 Skills：只列名称/简介、按需读 SKILL.md。它是候选项，不阻塞 v3；完整讲解与联网实践归 v4。完整交互保留在 main，v1 为符合 <400 行约束的单任务入口。v1/v2 继续非流式；基础 SSE 在 v3 单独展开，完整屏幕布局在 v6 展开；MCP、插件市场、多 agent 调度继续留作后续候选。

可观测性专题安排在 v3 配置完成后、v4 Skills 与联网之前，单独提交、写教程和验收，现有主版本编号保持。先保留 v3 的学习快照，再沿主线加入观测增量；全部生产 Rust 继续计入总行数。这样练习后续能力时，已经可以沿 trace 查看模型请求、工具反馈与上下文变化。

todo_write 与 ask_user_question 作为后期的常用工具能力分别加入。前者随 v4 的长任务出现，先做当前任务的结构化计划；后者随 v5 的会话状态与恢复加入，处理等待用户和继续执行。v6 完善两者的界面。每个工具独立提交、写教程和验收，v1–v3 的最小四工具范围保持。

## v3：基础流式与个人配置

基础 SSE 作为独立学习增量：显示文本事件，响应完整后继续原有工具循环，讲清显示状态与可回传历史的区别。随后逐步增加配置与模型目录，避免一次混入多种状态机制；继续控制约 1000 行的整体范围。

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

## Context 信息展示：v3 先做轻量信息，v6 完善界面

2026-10-14 纳入计划：借鉴 Codex / Claude Code 让上下文用量可见的体验，帮助学习者判断上下文在怎样增长、压缩何时发生。先在 v3 的滚动终端显示简短信息；v6 再接到状态栏与 `/context` 详情。这属于上下文机制的可观察性，继续遵循少量清晰代码的原则。

v3 结合模型目录与配置，先展示四类信息：

| 信息 | 来源与口径 |
|---|---|
| Context window | 当前 provider/model 的已知窗口容量，单位 tokens；来源为明确的模型元数据、预置或手动配置，未知则显示 unknown |
| Last input | 最近一次主模型请求的 API input_tokens；标明是上一请求的统计，不是当前待发送输入，也不累计整段会话的 usage |
| Request budget | 当前待发送请求的序列化字节数 / 本地字节上限；沿用现有预算，独立标注单位，不冒充 token 窗口占用率 |
| Compaction | 已发生的压缩次数、最近一次压缩前后字节数；压缩完成后按新输入更新显示 |

窗口容量与同口径 token 用量都已知时，可以展示已用/剩余百分比，标明对应哪次请求；若是对待发送输入的简单估算，必须显示 `~` 或 estimated。窗口未知、统计未返回或依据不一致时不生成精确百分比。窗口容量、本地请求预算和输出额度分别标注，避免把本地字节限制误解为模型的 context window。

沿用简单估算与已有 API usage，不为界面引入 tokenizer 或额外计量服务。首版只需一个简短文本输出，清楚展示模型窗口、上次输入、本地预算与压缩状态；不要求实时精确 token 计数。v6 的 `/context` 可进一步列出固定指令、用户任务、摘要与保留的工具交互组，并在常驻状态栏展示简要占用情况。

验收关注普通学习场景：连续读取文件后用量增长；压缩后当前输入体积下降；切换模型时窗口来源随之更新；缺少容量或 usage 时诚实显示 unknown。用可控 fixture 验证显示口径，不扩展成复杂监控或可靠性系统。

## v3 后的独立专题：Agent 可观测性

终端显示 `step 10 · working` 时，学习者已经能看见循环在推进。下一步会想知道：这一轮发给模型的历史是什么，模型为什么提出这次调用，工具反馈怎样进入下一轮，压缩又移除了哪些旧记录。这个专题把这些问题连成可复盘的执行轨迹。

从已有 Progress 和 journal 的事件语义出发，补齐模型开始/结束、工具开始/结束、压缩提交、错误与停止。用具体事件和小型 observer 接收生命周期 hook，关联 session、用户回合、step 与原始 call_id；执行、历史提交和停止仍由手写核心负责。终端消费进度，telemetry 适配器把观测事件映射为 spans。现有 journal 的写入契约独立保留。

一次用户任务对应一条 trace，多轮对话用 session_id 关联多条 trace。每次主模型和摘要调用各有自己的 generation，工具与请求它的 generation 同属相应 step。结构示意如下：

```text
session：一次多轮对话
└─ trace：本次用户任务
   ├─ step 1
   │  ├─ generation：主模型请求
   │  └─ tool：bash
   └─ step 2
      ├─ compaction
      │  └─ generation：摘要请求
      ├─ generation：主模型请求
      └─ tool：edit_file
```

当前 `step N · working` 在主循环开始时发出，表示进入第 N 轮；最终 `N steps` 统计成功取得主模型响应的轮数。计划分别记录循环轮次、主模型完成次数、工具调用次数和摘要请求次数。一个 step 可以有多个工具，压缩还可能增加模型请求；这些计数必须有明确含义。

telemetry 默认关闭，配置显式使能。以下是计划中的 TOML 接口：

```toml
[telemetry]
enabled = true
backend = "langfuse"
capture_content = false
```

关闭内容采集时记录模型名称、实际 usage、耗时、调用关联、工具结果状态、压缩前后字节数与停止原因。学习具体决策时可显式开启脱敏、限长后的模型输入/输出与工具参数/结果；密钥、认证头和 encrypted_content 始终排除。模型未返回的 usage 保持未知。模型给出最终回答与独立任务验证继续分别记录。

初始凭证沿用 `LANGFUSE_PUBLIC_KEY`、`LANGFUSE_SECRET_KEY`、`LANGFUSE_BASE_URL`。通过现有配置加载方式解析 cwd .env，进程值优先，再把选定值传给 exporter；变量存在不会自动开启上传。后续配置菜单沿用同一配置与凭证来源。凭证值不进入配置示例、日志或 trace。

Rust 接入采用原生 OpenTelemetry API 与 OTLP/HTTP exporter。2026-10-12 核查的已发布候选为 `opentelemetry`、`opentelemetry_sdk`、`opentelemetry-otlp` 0.33.1；实施时核对该固定版本与当前依赖的组合。Langfuse Server 版本和 Rust SDK 版本分别记录。按 v4 文档用项目公钥/密钥生成 Basic Auth，向 `/api/public/otel/v1/traces` 导出，并设置 `x-langfuse-ingestion-version: 4`；每个 span 完成后导出，session 等查询维度传播到相关子 span，call_id 作为元数据保留，span ID 由 OTel 生成。

网络导出使用有界后台队列，失败或丢弃数量清楚可见，遥测故障不改变工具执行与任务停止。结束和取消时做有时限的 flush；取消仍按核心契约记录结果未知。本地 JSONL 是执行记录，Langfuse 是可观测投影，恢复判断继续由 v5 的会话契约承担。

验收分两层：离线检查事件顺序、父子关系、call_id、开关、导出失败与有界退出；真实运行后从 Langfuse 读回 observations，核对多轮任务、工具故障反馈、强制压缩和取消轨迹是否与本地记录一致。终端可给出本次 trace 链接。2026-10-12 只确认了本地变量声明和官方协议，尚未验证账号、部署版本或实际入库。评估器与自动评分另作后续实验。

协议与层次依据：[Langfuse v4 自定义接入](https://langfuse.com/integrations/native/opentelemetry/migration-to-v4)、[轨迹组织建议](https://langfuse.com/docs/observability/best-practices)、[Rust OTLP 0.33.1](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/)。

## v4：工作方法、联网与任务计划

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

### todo_write：把长任务进度变成结构化状态

比较两个库、逐页核对资料、写报告并检查引用时，循环可能已经进入 step 10，仍有若干交付要求尚未处理。todo_write 让模型显式维护工作项，也让用户看见当前在做什么、还剩什么。复杂的多步骤任务默认考虑使用；短任务可以直接完成。在阶段推进或计划改变时更新，原用户目标与约束继续独立保留。

首版接收完整 Todo 列表，每项有稳定 id、content 和 pending/in_progress/completed/cancelled 状态；每次有效调用整体替换当前列表。列表项数、文本长度和总字节有界，校验失败或超限时保留原状态并通过原 call_id 返回工具错误。模型更新计划，Session 持有当前用户任务的最新版，终端先做简单列表展示。新任务初始化当前列表，旧任务的变更保留在 journal；/new 清空会话状态。

最新版 Todo 作为有预算的结构化任务状态进入模型输入，压缩旧历史时仍可提供它。工具调用与结果继续完整关联回传；上下文摘要与 Todo 状态各有自己的职责。todo.updated 事件关联任务、step 和 call_id，供终端与 Langfuse 展示。

step 反映主循环轮次，Todo 反映计划中的工作项。completed 是模型维护的状态声明，要与实际产物、测试或检查记录对应；它不替代独立任务验证。模型可以根据新发现调整计划和取消旧步骤，最终仍需核对用户要求。

离线验收覆盖有效更新、非法状态或重复 id 时不修改列表、/new 与新任务初始化，以及压缩后当前 Todo 保留；真实长任务检查计划变更、工具反馈、产物与验证记录能否对应。v4 先实现任务内状态，v5 再赋予跨进程恢复语义。

代表任务：比较两个库的官方 API，维护研究与验证 Todo，生成含原始 URL 的 Markdown 表格。验证来源、完成项和产物能对应结论，不要求固定措辞或固定调用顺序。

## v5：保存的是可解释的会话

JSONL 适合这个教学版本：每行一个事件，能追加、能逐行查看，也便于解释中断留下的痕迹。先做单进程、单写入者，不引入数据库和分支树。

文件头记录 schema_version、session_id、工作目录、provider/model 与开始时间；后续事件有递增 seq，记录用户任务、模型输出、tool_started、tool_finished、压缩提交与停止。凭证不写入记录。摘要是历史的一种投影，完整事件仍然保留。

从有效的 Todo 更新与闭合工具交互组重建当前任务列表，保留工作项 id、状态和所属任务。恢复后明确显示剩余工作，原工具结果和验证证据仍分别保留；不能从摘要中的推测生成已完成状态。

实现顺序：`sessions list/show` → 重建已完成组及最近一次有效摘要 → `resume <id>` 接收新任务 → 处理 Ctrl-C 与尾部半行。对于已发起的文件或外部操作，tool_started 而没有 tool_finished 表示结果未知：先检查文件或外部状态，再决定后续动作，不能直接重放写入与命令。明确记录的待答问题按下述问答契约恢复。

v2 的 `.crabloop/run-*.jsonl` 只用于观察一次运行，已经可以教“完整记录与模型输入不同”。v5 才赋予日志稳定 session 身份、版本和恢复语义，并移到 home 下；早期日志不承诺自动迁移为可恢复会话。

### ask_user_question：等待用户回答，再继续原任务

有时下一步依赖用户选择，例如一份接口改动要保留旧格式还是迁移。已有任务、代码与合理默认无法确定关键选择时，模型调用 ask_user_question。Harness 进入 WaitingForUser，记录待答问题；真实回答作为原 call_id 的 function_call_output 回传，然后继续原任务。等待期间不发送重复模型请求，当前 step 与待答状态分别展示。

首版只支持一个问题、少量可选候选项和自由文本，问题与回答内容都有大小上限。问题由 Harness 分配 question_id，并绑定 session、任务与可信工具调用；输入与回答分别记录。运行核心持有待答状态，终端负责收集回答，再把回答提交给对应问题。模型无需决定输入组件，工具执行器也不直接取得终端所有权。

交互入口先用简单 CLI 问答完成闭环。-p、管道或无交互输入的入口返回 NeedsInput，并保存待答状态；resume 后可以回答。空输入、EOF、取消或超时都有明确结果，不推断用户答案。校验重复、过期或不属于当前待答问题的回复。

暂停时保存原响应中的待答调用及组内已执行结果，恢复后只继续尚未完成的部分。已完成工具不会因用户回答或进程重启重新执行，结果未知的文件或外部操作继续遵循先检查的恢复规则。回答问题仍属于原用户任务，保留其 Todo、上下文锚点和调用预算；等待用户不增加模型请求次数。

question.requested、question.answered 和 question.cancelled 等事件进入 journal 与 telemetry，并在恢复验收中核对 trace 关联。离线测试确认真正等待、原 call_id 回传、非交互 NeedsInput、重复回答和取消；同轮先执行 write_file 再提问时，恢复不会再次写入。真实交互验证提问、答复、退出与恢复后继续同一任务。

## v6 建议：从基础流式长出完整终端界面

早期交互版已经用 rustyline 提供输入编辑和历史，用同一个 Session 处理多次提问，并显示工具进度。Ctrl-C 取消后显式要求检查文件和 /new；当前没有恢复语义。基础文字增量安排在 v3；v6 在其上分步增加内容渲染、命令交互与布局，并接入 v5 的中断/恢复协议。

v6 拆成三个可运行、独立验收的增量：

| 增量 | 范围 | 需要观察的行为 |
|---|---|---|
| 输出渲染 | You/CrabLoop 角色区分；Progress、可读推理与工具分区；Markdown；Context 用量、压缩、Todo 与待答状态 | 中文换行和终端宽度正确，任务进度与循环轮次容易区分；先在滚动终端中完成 |
| 输入与 slash command 交互 | 命令说明、过滤、键盘选择与补全；复用选择与文本输入能力回答待答问题 | 候选选择清楚可见，参数与未知命令有提示；回答绑定原 question_id，提交后继续原任务 |
| 完整布局与焦点管理 | 对话区、输入区、状态栏、滚动、详情展开、焦点和增量刷新 | 模型等待期间按键仍可响应，窗口缩放、中文粘贴、多行输入、取消与退出恢复正常 |

thinking 状态表示运行进度；推理正文只展示供应商实际提供的可读 reasoning/summary。encrypted_content 继续保持不透明。commentary、final_answer 和推理内容分别呈现，完整协议项仍用于历史回传；最终答案混入 commentary 的问题已在 [v1 GA 收尾](../evidence/v1/ga-review.md)修复，main 已加入按轮显示 commentary 的轻量增量：过程说明先于本轮工具，最终答案单独显示，-p 将过程写到 stderr；已有学习分支保持原样。流式 Markdown 需要处理稳定内容与未完成尾部。实时 bash stdout/stderr 还需执行层的输出事件，作为明确增量实现。

### 对话与执行过程的视觉区分（未来 Todo）

2026-10-14 根据实际终端截图补充：当前输入提示与 Agent 标题共用青色，正文、工具与状态行缺乏层次，Markdown 标记直接显示。目标是让读者扫一眼就知道“谁说的、正在做什么、工具实际返回了什么、最终回答在哪里”。此项列为 v6 输出渲染的首个增量，先改善滚动终端，不要求完整屏幕布局；v3 仍专注基础 SSE、配置与轻量 Context 信息。

采用明确标签、留白、缩进和少量语义颜色。固定 UI 文案使用英文，用户与模型内容保留原语言。颜色集中在标题和状态，长段正文使用终端默认前景色；不把整段推理压成低对比灰字。以下色彩为候选，实施时在深浅终端主题中检查，不固定背景色，也不引入主题编辑器。

| 内容 | 标签与视觉层次 | 首版行为 |
|---|---|---|
| 用户问题 | `You`，蓝色粗体标题；与上一轮留空行 | 输入提示使用 `You >`；多行续行对齐，提交后保留一个清楚的用户块，不重复回显 |
| Agent 回答 | `CrabLoop`，青色粗体标题；正文正常对比度 | 单独段落，支持基础 Markdown；与过程、工具块留空行 |
| 过程说明 | `Progress`，次级标题与缩进 | 展示模型实际返回的 commentary，默认可见；不将其标成内部思考 |
| 可读推理 | `Reasoning` 或 `Reasoning summary`，紫色次级标题，按实际内容类型命名 | 仅在 API 确实提供可读内容时出现；摘要明确标成 summary；密文不显示、不解码，没有内容就不生成空块 |
| 工具调用与结果 | `Tool · bash` / `Tool · read_file`，缩进显示命令或路径 | 每次调用与自身结果相邻；显示 `Running`、`Done`、`Failed`、`Timed out` 或 `Cancelled`，以真实结果为准 |
| 运行状态 | `Step 2/16 · Working…`，紧凑次级行 | 等待状态不冒充推理正文；工具成功不等于任务完成，最终停止原因另行显示 |

`Done` 可用绿色，`Failed` 用红色，超时或取消用黄色，但始终保留文字。`NO_COLOR` 或非 TTY 输出去掉颜色和光标控制，仍能依靠标签、缩进和空行区分。`-p` 的最终回答继续写 stdout，过程和工具轨迹写 stderr；展示样式不写入模型历史或 journal。

工具默认显示名称、参数摘要、执行结果和少量输出预览；失败时直接显示退出码或原因，长输出明确标注预览/截断。首个滚动终端增量不假装支持折叠，完整布局阶段再按调用展开参数、stdout/stderr 和详细结果。模型反馈继续使用原始工具结果，UI 预览不改变它。

下面只是排版示意，不代表实际执行证据；Reasoning summary 仅在 API 提供时出现：

```text
You > 检查测试失败的原因

  Step 1/16 · Working…
  Progress
    我先运行测试，确认失败位置。

  Reasoning summary
    从失败用例定位相关实现。

  Tool · bash · Done · exit 0
    cargo test --locked
    test result: ok. 3 passed

  Step 2/16 · Working…

CrabLoop
当前测试通过，未复现失败。请提供触发问题的输入。

  Final · 2 steps
```

基础 Markdown 先覆盖段落、标题、列表、引用、粗体、行内代码和代码块。用户输入与工具输出按字面文本显示，避免命令或日志被当成 Markdown 改写；代码块保留缩进，链接保留可读目标。复杂表格、语法主题和动画不作为首版门槛。流式回答只刷新未稳定的尾部，完整段落提交一次，避免重复标题、重复文本和频繁跳动。

未来待办按顺序完成，每项独立检查：

- [ ] **v6.1a：角色与内容标签。** 在滚动终端区分 You、CrabLoop、Progress、可读 Reasoning 与 Tool；加入空行、缩进和少量颜色，保留 Working/停止状态。
- [ ] **v6.1b：回答正文渲染。** 支持基础 Markdown、中文换行和代码缩进；接入 v3 流式事件，稳定内容只提交一次。
- [ ] **v6.1c：工具与 Context 摘要。** 关联每次调用和结果，显示失败原因与输出预览；沿用 Context 容量、上次请求 usage、本地字节预算及压缩信息的既定口径。
- [ ] **v6.2 / v6.3：交互与布局。** 沿用下文 slash 菜单与完整布局安排，增加工具/推理详情展开及状态栏；收起只影响显示，不删除记录或模型上下文。

用同一组受控对话样本检查：普通问答、commentary → 工具 → 最终回答、可读推理摘要、工具失败，以及中文和代码块；再在深色、浅色、无颜色和窄终端中人工查看。基本通过标准是能准确辨认用户/Agent、过程/答案、调用/结果，内容不重复，脚本输出不含展示控制码。按增量补少量渲染检查和一次 PTY 验证，不扩展成完整主题系统或大规模 UI 测试工程。

### 命令交互、布局与实现边界

现有 /new、/help、/exit 已有本地执行逻辑。未来 /model、/config、/skills、/resume 的功能随配置、Skills、会话版本实现，v6 菜单提供统一入口。命令的可用状态和取消行为遵循同一个运行核心；配置菜单与手改 TOML 使用同一来源。

slash 菜单仍遵循输入 / 展示候选、↑↓ 选择、Tab 补全、Enter 提交、Esc 关闭并保留草稿。问题选项与自由文本回答调用 v5 的问答契约；UI 展示待答内容与状态，运行核心决定何时继续。完整布局增量再增加 Todo 常驻区域、详情与问答焦点管理。

终端库按实际问题选择：

| 时点与用途 | 建议组件 | CrabLoop 自己负责什么 |
|---|---|---|
| 当前轻量终端聊天 | rustyline 18.0.1，已实现 | 输入编辑与进程内输入历史；Session 管理模型历史 |
| v4 的 config 菜单 | dialoguer：选择、输入、隐藏密码输入 | 配置校验、优先级、存储与生效来源 |
| v6 的完整 TUI | Ratatui：布局、段落、列表、滚动与状态栏 | 会话视图、焦点、事件处理与状态更新 |
| 终端输入和终端模式 | Crossterm | 输入事件怎样变成提交、取消或选择动作 |
| 多行输入框 | 选与 Ratatui 版本兼容的文本编辑组件；如维护中的 tui-textarea-2 候选 | 提交快捷键、粘贴体验、与当前任务的关系 |

当前官方资料提供这些基础能力：[dialoguer](https://docs.rs/dialoguer/latest/dialoguer/)、[Ratatui](https://ratatui.rs/)、[Crossterm](https://docs.rs/crossterm/latest/crossterm/)。文本输入组件的维护状态与依赖版本在实施时复核；这里不固定尚未构建验证的组合。

Codex CLI 的 Rust TUI 作为具体源码参考。本次只读检查本机 openai/codex 仓库的 `279ba894152b2c01c5294cc0723b463b209bdca4` 快照，manifest 与 Cargo.lock 对应 Ratatui 0.30.2、Crossterm 0.29.0、ratatui-macros 0.7.2。Crossterm 实际通过 patch 锁定 OpenAI fork 的 `45fecb9508105988f42fe6ff0441783ed3717f92`；这些是该源码快照的依赖，不当作 CrabLoop 将来必须照抄的版本。

阅读顺序：[依赖声明](https://github.com/openai/codex/blob/279ba894152b2c01c5294cc0723b463b209bdca4/codex-rs/tui/Cargo.toml#L82) → [终端与输入事件](https://github.com/openai/codex/blob/279ba894152b2c01c5294cc0723b463b209bdca4/codex-rs/tui/src/tui.rs) → [自有 TextArea](https://github.com/openai/codex/blob/279ba894152b2c01c5294cc0723b463b209bdca4/codex-rs/tui/src/bottom_pane/textarea.rs) / [ChatComposer](https://github.com/openai/codex/blob/279ba894152b2c01c5294cc0723b463b209bdca4/codex-rs/tui/src/bottom_pane/chat_composer.rs) → [流式显示控制器](https://github.com/openai/codex/blob/279ba894152b2c01c5294cc0723b463b209bdca4/codex-rs/tui/src/streaming/controller.rs)。源码显示 Codex 在基础库之上持有编辑与渲染状态；依赖中另有 pulldown-cmark、textwrap、Unicode 处理及 insta 快照测试。

CrabLoop 优先借鉴终端退出恢复、输入/粘贴与提交的区分、流式内容的稳定部分和可更新尾部，以及小组件快照测试。完整布局增量先用 Ratatui/Crossterm 发布版完成最小界面，输入框先选兼容组件；需要自有交互行为时再读 Codex 对应实现。只有遇到具体问题时才评估 fork 或复杂组件，保持核心与界面职责清楚。

当前核心通过 Progress 回调发出模型轮次和工具开始/结束，交互与单次入口共享 Session::turn。v6 复用可观测性专题的生命周期与关联 ID，再按需要增加 channel 上的 TextDelta、可读推理和 ToolOutputDelta。CLI/TUI 消费事件并展示，核心负责工具执行；模型网络等待不阻塞按键。当前单次 CLI 入口仍可用于脚本和验收；TUI 复用同一个运行核心。可以在具体需要出现时拆出 src/ui/，所有生产 Rust 仍计入总行数，另列核心/界面分项帮助阅读。

教学验收要求学习者能从界面追踪主循环、工具反馈、压缩和停止，并找到对应代码位置。保留可见的当前 step、轮次预算与 Context 信息，工具和摘要调用另行计数；状态栏与 Langfuse 的同一任务、step、call_id 能对应。输出渲染、slash 交互和完整布局分别验证，主题与复杂快捷键随后按需要添加。

### 学习参考：llm-space 的工具契约与交互

llm-space 是工具与交互的学习参考之一。本次只读查看本地源码快照 e65eea24c87ee5b91618b7eb50e2af77a30ef042：todo_write 的调用参数携带完整列表，执行入口返回 OK，专用视图呈现各项状态；ask_user_question 标记为需要人工结果，排除在自动执行路径之外。研究示例展示了创建、执行和调整计划的工作方式。

学习重点是模型可见的工具契约、执行与显示的职责、长任务进度以及人机交互的状态边界。CrabLoop 按自己的 Rust 学习目标实现具体状态与函数，保留手写 loop，再用任务、上下文、恢复和界面增量逐步验证。当前参考属于静态源码检查，未运行该应用作对照验收。

固定源码入口：[工具定义与执行](https://github.com/deer-flow/llm-space/blob/e65eea24c87ee5b91618b7eb50e2af77a30ef042/packages/runtime/src/tools/built-in/misc.ts)、[自动执行边界](https://github.com/deer-flow/llm-space/blob/e65eea24c87ee5b91618b7eb50e2af77a30ef042/packages/core/src/types/tools/index.ts)、[Todo 展示](https://github.com/deer-flow/llm-space/blob/e65eea24c87ee5b91618b7eb50e2af77a30ef042/packages/ui/src/components/thread-playground/message/todo-write-view.tsx)、[研究示例](https://github.com/deer-flow/llm-space/blob/e65eea24c87ee5b91618b7eb50e2af77a30ef042/packages/ui/src/components/thread-playground/examples/deep-research.md)。

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

Rig 是可跳过的高阶对照项，v1–v7 主线可以沿手写实现独立完成。它的官方文档覆盖模型/provider、工具、流式、embedding、向量存储与 memory，并允许直接调用模型、保留自有 loop；也提供 agent runtime。[Rig 官方介绍](https://rig.rs/docs)、[API 入口](https://docs.rs/rig/latest/rig/)

建议先在独立的实验开发分支里接一个 Rig 模型后端，保留 CrabLoop 的循环、上下文策略、工具执行与 session/memory 规则。用相同 fixture 和代表任务比较原手写 HTTP 与 Rig 适配：调用/结果匹配、reasoning 与 phase 回传、流式中断、usage/错误、工具副作用和预算是否仍有相同含义。尤其不能因为 API 都叫 Responses 就假定原生字段一定完整往返。

出现重复的多协议 provider 接入、embedding 等具体需要时，再评估 Rig 能减少的维护工作。实验固定实际采用的已发布版本并阅读对应源码/API，记录依赖变化、适配代码和调试成本，解释哪些重复代码减少了、哪些状态仍由 harness 持有。通过协议与任务对照后决定是否采用；使用 Rig agent runtime 是第二个独立对照题，不与替换模型客户端同时发生。

Rig 的当前 API 也有图像生成接口。如果之后希望 CrabLoop 增加生图任务，可以单独规划 generate_image 工具与产物文件管理；它与引入通用 agent 基础库分开决定，不影响当前 Responses 主循环路线。

这些建议的目的，是在已经理解 harness 状态之后，学习如何选择和约束第三方抽象。终端布局与输入组件可以较早复用；模型执行与记忆策略先保持可观察，再用实验决定复用边界。

## 教程与提交继续沿用同一条路

仓库继续使用 `tutorial/vN/` 组织教学，`docs/devlog/` 留下真实经过，`docs/` 放设计、路线与证据解释。每版新增文件时再建相应目录。每个通过检查的可验证增量提交、合入并推送 main；main 保存最新可运行内容和完整提交历史。每个完成版本保留 `milestone/vN` 学习分支及不移动的 `vN` 标签，v6/v7 也沿用；后续开发另开分支，学习分支保持稳定。阶段文档明确哪些已实现，哪些仍是计划。

main 与学习分支的具体关系见 [Git 工作流](git-workflow.md)；历史版本不复制到根目录。
