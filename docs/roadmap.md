# CrabLoop：从一轮行动，长到自己的工作台

更新：2026-10-11。v1/v2 已按[实施计划](plan.md)完成；v3–v5 是后续规划，本文中的配置和命令尚未实现。

最值得保留的是逐步生长的节奏。先看见模型调用一次工具，再看见历史变长，最后才遇到“每次都要设置环境”“换模型不方便”“明天怎样接着做”。每版解决一个能亲手观察的问题。

## 五个版本的主线

| 版本 | 范围预算 | 核心问题 | 内容与一个代表任务 |
|---|---:|---|---|
| v1 | 约 300 行；350–360 也可接受 | 模型怎样行动并依据反馈调整？ | Responses + read_file/write_file/edit_file/bash；修复函数，再做一次多文件资料整理 |
| v2 | 约 500 行，允许合理浮动 | 历史变长后，下一轮看到什么？ | 根 AGENTS.md、完整交互组、输入预算、摘要、JSONL 运行记录；小预算下完成任务 |
| v3 | 约 1000 行，优先保持可读 | 怎样成为不用反复配置的个人 CLI？ | `~/.crabloop`、TOML、凭证、provider 预置/Custom、模型目录与缓存；换 provider 执行同一任务 |
| v4 | 先按功能估算，不预设硬上限 | 怎样复用工作方法，并查阅外部资料？ | Skills 最小支持、web_fetch、web_search、CLI 配置菜单；查官方资料，保存带来源的比较报告 |
| v5 | 先按功能估算，不预设硬上限 | 进程退出后，怎样知道做到哪里并继续？ | JSONL sessions、list/show/resume、压缩状态重建、取消与未完成操作；中断后核对并继续 |

行数用于提醒我们收住范围，统计全部 src/**/*.rs，测试、教程与开发脚本单独记录。不会为了从 360 降到 300 而压缩难读的代码，也不会因为模型多输出一个正确的统计字段就判失败。验收看任务事实、工具结果和程序的关键边界。

v3 若完成配置与目录后仍有余量，可以提前加入最小 Skills：只列名称/简介、按需读 SKILL.md。它是候选项，不阻塞 v3；完整讲解与联网实践归 v4。Streaming、MCP、插件市场、复杂 TUI 暂不挤入主线。

## v3：先给程序一个家

程序代码仍是仓库根目录的一个 Cargo 工程。用户数据放在自己的 home，默认布局如下；目录随对应版本出现，不提前创建空功能。

```text
~/.crabloop/
├── config.toml                 # v3：provider、默认模型、预算等普通设置
├── credentials.toml            # v3：本机凭证，单独保存
├── cache/
│   └── models-<profile>.json    # v3：可重建的模型目录
├── skills/<name>/SKILL.md       # v4：用户安装的工作方法
└── sessions/<session-id>.jsonl  # v5：跨进程会话
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

## 教程与提交继续沿用同一条路

仓库继续使用 `tutorial/v1/` 到 `tutorial/v5/` 组织教学，`docs/devlog/` 留下真实经过，`docs/` 放设计、路线与证据解释。每版新增文件时再建相应目录。每个可验证增量提交并推送。每个完成版本保留 `milestone/vN` 学习分支及不移动的 `vN` 标签，v6/v7 也沿用；后续开发另开分支，学习分支保持稳定。阶段文档明确哪些已实现，哪些仍是计划。
