# CrabLoop 需求 Todo

更新：2026-10-14。这里统一记录后续需求、排期与完成状态；[roadmap](roadmap.md) 解释设计、版本边界与验收。此列表是仓库开发待办，与 Agent 运行时的 todo_write 工具不同。

## 维护方式

- 新需求先记一条：稳定编号、简短描述、目标迭代（或“待排期”）、必要的设计链接。不要求每个小改动先写设计文档。
- `[ ]` 表示未完成；开始实施时加“进行中”；实现并验证后才改为 `[x]`，附提交或验收链接。记录需求、完成设计、修改文档都不等于功能完成。
- 目标迭代允许调整；小改动可单独排期，不必等完整大版本。候选项需先确定范围，不能自动成为当前迭代的承诺。取消项标注“取消”和原因，不用完成勾选代替。
- 需求状态只在本文件维护；roadmap 保留详细设计和验收，devlog 保留实施经过。排期或范围变化时同步相关说明，编号不复用。已完成条目可移到文末归档。
- 保持学习优先与少量清晰代码；v2–v3 不因通用生产标准新增重试、复杂恢复或 tokenizer。历史 v1/v2 实施记录继续留在原计划与证据中，不重复抄进本表。

## 已排迭代

以下是计划目标，不表示已经开始实施；v6–v7 延续路线图中的建议排期。

- [ ] **REQ-001 · v3：基础 SSE。** 文本增量可见，完整响应确认后再执行工具，区分显示状态与协议历史。[设计](roadmap.md#v3基础流式与个人配置)
- [ ] **REQ-002 · v3：个人配置。** ~/.crabloop、TOML、独立凭证、provider presets/Custom 与明确配置来源。[设计](roadmap.md#v3基础流式与个人配置)
- [ ] **REQ-003 · v3：模型目录与缓存。** 发现、补充和选择模型，区分元信息来源。[设计](roadmap.md#模型目录发现补充选择)
- [ ] **REQ-004 · v3：轻量 Context 信息。** 窗口容量、上次主请求 input tokens、本地字节预算、压缩次数与前后体积；未知或估算明确标记。[设计](roadmap.md#context-信息展示v3-先做轻量信息v6-完善界面)
- [ ] **REQ-026 · v3：显式运行上下文。** 通过 `<system-reminder>` 注入工具实际工作目录、请求时间快照（UTC）、平台和 Bash；与身份/项目规则分开，计入预算、不重复追加历史。普通目录可直接启动；Project 管理界面未排期。[设计](roadmap.md#运行上下文与工作目录身份之外的环境事实)
- [ ] **REQ-005 · v3 后独立专题：可选 telemetry。** 生命周期 observer、关联 ID、Rust OpenTelemetry/OTLP → Langfuse；单独验收。[设计](roadmap.md#v3-后的独立专题agent-可观测性)
- [ ] **REQ-006 · v4：最小 Skills。** 目录摘要与按需加载正文；v3 有预算余量时可提前最小子集，不作为 v3 门槛。[设计](roadmap.md#v4工作方法联网与任务计划)
- [ ] **REQ-007 · v4：联网工具。** Tavily Search / Firecrawl Scrape，对模型提供 web_search/web_fetch。[设计](roadmap.md#v4工作方法联网与任务计划)
- [ ] **REQ-008 · v4：配置菜单。** TOML 编辑与菜单同源，按需导入选中的 .env 配置，显示来源。[设计](roadmap.md#菜单与手工配置使用同一套文件)
- [ ] **REQ-009 · v4：todo_write。** 维护任务内结构化计划，区分 Todo 完成声明与实际验证。[设计](roadmap.md#todo_write把长任务进度变成结构化状态)
- [ ] **REQ-010 · v5：会话与 Todo 恢复。** JSONL sessions、list/show/resume，以及中断后的已知状态边界。[设计](roadmap.md#v5保存的是可解释的会话)
- [ ] **REQ-011 · v5：ask_user_question。** 保存待答问题、接收真实回答并继续原任务。[设计](roadmap.md#ask_user_question等待用户回答再继续原任务)
- [ ] **REQ-012 · v6.1a：角色与过程区分。** 用户消息使用低饱和度背景块；Agent 正文正常亮度；Reasoning / Reasoning summary 与 Progress 使用可读的次级灰色，保留不同标签；工具块独立分组。[设计](roadmap.md#对话与执行过程的视觉区分未来-todo)
- [ ] **REQ-013 · v6.1b：回答 Markdown。** 基础排版、中文换行、代码缩进；流式稳定内容不重复输出。[设计](roadmap.md#对话与执行过程的视觉区分未来-todo)
- [ ] **REQ-014 · v6.1c：工具与 Context 摘要。** 调用/结果配对、状态、失败原因和输出预览；展示 Context 与压缩信息。[设计](roadmap.md#对话与执行过程的视觉区分未来-todo)
- [ ] **REQ-015 · v6.2：输入与 slash 交互。** 候选菜单、键盘补全、Todo/问题交互，保持既有核心语义。[设计](roadmap.md#命令交互布局与实现边界)
- [ ] **REQ-016 · v6.3：完整终端布局。** 对话区、焦点、滚动与详情展开；底部输入区用上下细横线、`crab>` 与青色焦点强调，支持多行；历史用户消息保留 `You` 标签与背景块；紧凑状态栏及 /context 详情沿用已有口径。[输入区设计](roadmap.md#输入区清楚的边界与轻量辨识度) · [布局](roadmap.md#命令交互布局与实现边界) · [Context](roadmap.md#context-信息展示v3-先做轻量信息v6-完善界面)
- [ ] **REQ-017 · v7：可解释的 memory。** 显式写入、有预算的检索、来源、修正与遗忘。[设计](roadmap.md#v7-建议记住可复用的经验)
- [ ] **REQ-018 · v8 选修：Rig 对照。** 独立实验比较后端适配，主线学习不依赖它。[设计](roadmap.md#v8-选修把手写实现与-rig-放在一起比较)

## 待排期／候选

这里接收临时想到的小改动和范围尚未确定的想法；先记录，明确学习收益与增量后再排期。明确需求可直接记录在这里，复杂设计再链接 roadmap；不为填满列表新增需求。

- [ ] **REQ-019 · 待排期：Bash 实时输出。** 需要执行层提供 stdout/stderr 增量事件；与现有工具开始/结束和结果预览分开实现。[边界](roadmap.md#v6-建议从基础流式长出完整终端界面)
- [ ] **REQ-020 · 候选，待决定：生图工具。** generate_image 与产物文件管理单独评估，不以引入 Rig 为前提，不承诺进入主线。[来源](roadmap.md#v8-选修把手写实现与-rig-放在一起比较)

- [ ] **REQ-021 · 待排期，小改动：版本命令。** 支持 `crabloop version`、`--version` / `-V`，stdout 单行输出 `crabloop <Cargo package version>` 并以 0 退出；版本来自 CARGO_PKG_VERSION，无需 API key，不加载模型配置、不创建会话或发请求。包版本与 v1/v2 教学阶段分开解释；首版不增加 Git SHA/build.rs。将 version 保留为命令，任务文本仍可用 `-p "version"` 显式传入。验收无凭证、无模型请求及三个入口输出一致。
- [ ] **REQ-022 · 待排期：自动 Release。** 有二进制分发需要时再确定发行版本命名、平台和发布流程；历史教学 tag 不自动批量发布，不增加额外 v1-ga 入口。
- [ ] **REQ-023 · 进行中，独立外观改动：GitHub 首页。** 一行徽章与导航；吉祥物和 Banner 已生成七种 Banner 风格候选，等待选定。取消循环箭头装饰，最终只将选定素材接入 README。
- [x] **REQ-024 · 已完成，独立工程改动：基础 CI。** main push / PR / 手动触发，单 Linux job 跑 fmt、clippy、Rust 与 Python 离线测试；不接模型密钥，不自动发布。实现：`0cab0db`；[首次 GitHub CI 通过](https://github.com/eriklee1895/crabloop/actions/runs/38520501082)。

- [ ] **REQ-027 · 待排期，独立 CLI Help 迭代。** 参考用户提供的 Cargo/Codex 截图，设计用途、Usage、分组、对齐、高亮与多色；窄终端和无颜色输出仍清晰；帮助无需凭证、模型配置或会话，不请求模型；参数与帮助来自同一份声明。v1 不承担该功能，不必等完整 TUI。[设计](cli-help-plan.md)

## 已完成的独立改动

- [x] **REQ-025：CrabLoop 身份与工作方式。** 系统提示词以 `You are CrabLoop 🦀` 开头，明确本地代码与资料任务、跟随用户语言、如实验证和完成后停止；问候与自我介绍直接回答。保持短段落，不加 header/XML；仅修改 main。[实施与验证](devlog/24-crabloop-identity.md)

- [x] **REQ-028：v1 收敛为 stdin 单任务入口。** 删除任务参数与完整 Help，README/.env.example 提供配置说明，保留明确错误。同步脚本、教程和七步重建；388 代码行 / 402 物理行，42 Rust + 5 Python 通过。实现 `97723d8`，学习分支 `388edc2`；main 运行入口保留。[实施与验证](devlog/26-v1-stdin-cli.md)
