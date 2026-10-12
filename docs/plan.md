# CrabLoop v1 → v2 Implementation Plan

> 版本说明（2026-10-12）：本文记录原始 v1 → v2 设计。当前 v1 学习分支按用户的新要求增加 `-p` 和多轮终端入口；完整内存历史仍保留 v1 的边界，不带压缩和 journal。以[交互补篇](../tutorial/v1/06-conversation.md)和[本次记录](devlog/06-v1-interactive.md)说明的变更为准。

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让学习者亲手用约 300 行 Rust 实现四工具 coding agent，再增加约 200 行上下文管理与压缩，掌握可迁移的 harness 基础。

**Architecture:** 一个 CLI 任务对应一个 Run，使用 OpenAI 原生 Responses API，并显式维护可回传历史。v1 完成模型—工具反馈循环；v2 在相同 loop 和工具之上增加完整记录、请求投影、预算和摘要。主模型、工具、上下文三个边界用具体函数连接。

**Tech Stack:** Rust 1.94.0 / edition 2024；reqwest 0.13.5、tokio 1.53.2、serde 1.0.229、serde_json 1.0.152、anyhow 1.0.104；Bash；原生 Responses function calling。

**Spec:** [CrabLoop 设计说明](design.md)。先阅读该说明；本文里的新增文件和命令均是待实施的安排。

**Execution:** 用户于 2026-10-11 授权设置 goal 并开始实施。由助手在本会话顺序构建参考实现、教程与开发记录；每个可验收增量同步 AGENTS.md 和进度。主用 DeepSeek V4.1 Flash，备选 gpt-6-luna。用户已授权每个通过验证的阶段提交并推送 GitHub。后续 v3–v5、配置/模型目录、Skills/联网和 sessions 补充见[路线图](roadmap.md)。

## Global Constraints

- 模型接入使用 OpenAI 原生 Responses API。公开工具名固定为 read_file、write_file、edit_file、bash。
- 每次请求由客户端显式发送 input；store 为 false，不传 previous_response_id 或 conversation。
- Rust 1.94.0，edition 2024；依赖按设计说明固定精确版本，提交 Cargo.lock 后使用 locked 构建。
- 主模型轮数 16；HTTP 超时 60 秒；Bash 超时 30 秒；max_output_tokens 为 2,048。
- 文件上限 65,536 字节，read_file 输出上限 8,192 字节，Bash stdout/stderr 各捕获最多 4,096 原始字节。
- v1 请求上限 131,072 字节；v2 默认 49,152 字节、触发阈值 36,864 字节、保留最近 2 组，摘要上限 min(4,096，hard/8)。
- v2 的 CRABLOOP_CONTEXT_BYTES 最小 4,096；根目录 AGENTS.md 上限 8,192 UTF-8 字节。
- v1 目标约 300 行，350–360 也可接受；v2 目标约 500 行，允许合理浮动；全部 src/**/*.rs 纳入计数，保持正常 cargo fmt，不牺牲必要错误处理。
- 先完成 v1 并保留 tag，再扩展 v2；不在两版中添加设计说明列出的选修能力。

## Review Focus

1. 同一响应里有两个调用，或调用 id 缺失/重复：call_id 匹配正确，非法协议不执行工具。归 Task 1、4。
2. reasoning/密文/phase、中间文本、incomplete：回传项完整，中间说明不误报最终结束，半截输出不执行。归 Task 1、4。
3. 中文、空 old_text、重复匹配、文件超限：截断不破坏 UTF-8，失败编辑不修改文件。归 Task 2。
4. stdout/stderr 大量输出或直接子进程超时：采集本身受限，不堵管道，直接子进程被回收。归 Task 3。
5. 压缩请求超限、摘要失败或失真：不提交失败输入，不删原记录，不把未验证写成已通过。归 Task 6、7、8。

## 学习路径：10 个站点

| 站点 | 问题与亲手练习 | Rust 重点 | 运行后应看见的证据 |
|---|---|---|---|
| 00 状态地图 | 手写一次修复任务的消息序列 | struct/enum 的职责 | 能区分 Run、Model turn、call、result |
| 01 原生模型协议 | 发请求，识别文本、调用、reasoning | serde、Value、Result、async | 请求与回传项正确 |
| 02 三个文件工具 | 读、写、精确替换；故意匹配失败 | Path、借用、字符串与错误 | 错误没有造成文件变化 |
| 03 Bash 工具 | 运行测试，制造非零退出/超时/大输出 | Child 生命周期、并发读管道 | 真实退出码与有限输出 |
| 04 行动循环 | 串行执行，再把结果送回模型 | 所有权、循环与停止原因 | 至少两轮，调用与结果关联正确 |
| 05 v1 综合任务 | 在小 Rust 项目修复 bug | 从协议到实际执行 | 失败测试 → 修复 → 独立验收通过 |
| 06 上下文投影 | 固定任务、AGENTS.md、JSONL、预算 | 状态持有者、序列化、文件追加 | 能检查每轮输入如何组成 |
| 07 摘要压缩 | 小预算触发；失败时不更新历史 | 先构造候选，再提交状态 | 输入变小，协议仍完整 |
| 08 压缩对照 | 完整历史、整组裁剪、摘要三个方案 | 纯函数测试与实验记录 | 区分协议正确与事实保留 |
| 09 v2 复盘 | 强制压缩后完成任务，再从空文件重建 loop | 迁移与解释能力 | 能沿记录解释每次继续和停止 |

每站围绕当前已经能运行的程序，耐心讲清“为什么需要下一步”。用连贯的中文叙述串起问题、小段代码增量、运行观察、故障与复盘；原理和 Rust 细节在用到时解释。材料需要覆盖原理、状态变化、练习、故障实验、验收和复盘，不强制套用相同的标题模板。具体遵循[教程写作约定](writing-guide.md)，语气参考[教程入口](../tutorial/README.md)。

每完成一个可验收的增量，在 docs/devlog/ 更新对应编号的开发记录，保留实际问题、取舍、验证结果与提交。下表对应的是待创建的教学文档，章节随实现逐步完成。

## 文件职责与共享接口

| 文件，均为未来实现 | 职责 |
|---|---|
| src/main.rs | 参数/环境入口、输出报告、进程退出码 |
| src/lib.rs | Config/Limits、RunReport、主 loop、公开测试入口 |
| src/model.rs | 原生请求 body、响应校验、可重放 output、call/text 提取 |
| src/tools.rs | 四个 strict schemas、参数解码、文件/Bash 执行、输出限制 |
| src/context.rs，仅 v2 | 固定任务、完整 Round、预算、候选输入和摘要提交 |
| src/journal.rs，仅 v2 | JSONL 事件追加与 flush，不实现恢复 |
| tests/common/mod.rs | 本地 fake Responses、临时目录和测试配置 |
| tests/fixtures/responses/*.json | 文本、多个调用、reasoning、phase、incomplete 等协议样本 |
| tests/protocol.rs、tools.rs、loop.rs、context.rs、compaction.rs | 分别验证这些边界 |
| tutorial/v1/00-state-map.md 至 05-v1-coding-task.md | 第一版的逐步教程 |
| tutorial/v2/06-context-projection.md 至 09-v2-retrospective.md | 在第一版上继续生长的教程 |
| docs/devlog/00-bootstrap.md 至 09-v2-retrospective.md | 实际开发经过、取舍、验证与提交 |
| fixtures/buggy-rust/、fixtures/verifier/ | 可修改的小项目和独立验收测试 |
| scripts/count-lines.py、verify-playground.sh | 规模报告与独立验收 |
| evidence/v1/、evidence/v2/ | 脱敏真实运行记录、差异、测试结果、复盘 |

共同类型：ToolCall { call_id: String, name: String, arguments: String }；ModelReply { output: Vec<Value>, calls: Vec<ToolCall>, text: String, is_final: bool, usage: Option<Value> }；ToolResult { call_id: String, output: Value }；StopReason 为 Final / MaxTurns / ContextLimit；RunReport { reason: StopReason, final_text: String, model_turns: usize }。工具输出中的 ok 为 bool。

Config { workspace: PathBuf, task: String, endpoint: String, model: String, api_key: String, limits: Limits }。Limits { max_turns: usize, http_timeout: Duration, bash_timeout: Duration, request_bytes: usize, file_bytes: usize, read_output_bytes: usize, stream_bytes: usize, max_output_tokens: u32 } 的默认值逐项对应设计说明。

tests/common 提供 StubResponses::start(replies: Vec<Value>) -> Self、endpoint(&self) -> &str、requests(&self) -> Vec<Value>、fixture(name: &str) -> Value、test_config(workspace: &Path, endpoint: &str) -> Config、make_call(call_id: &str, name: &str, arguments: Value) -> ToolCall。测试直接构造 Config；需测试环境变量时启动单独子进程，不在并发测试中修改全局 env。

## Task 1: 原生 Responses 接入与可重放协议，站点 00–01

**Files:** Create Cargo.toml、Cargo.lock、rust-toolchain.toml、src/main.rs、src/lib.rs、src/model.rs、tests/common/mod.rs、tests/protocol.rs、tests/fixtures/responses/{text,two_calls,reasoning,commentary,incomplete,bad_call_id}.json、tutorial/v1/00-state-map.md、tutorial/v1/01-responses.md、docs/devlog/01-responses.md；Modify README.md、.gitignore。

**Interfaces:** Produces Config::from_args_env(args: &[String]) -> Result<Config>；ModelClient::new(endpoint: &str, model: &str, api_key: &str, limits: &Limits) -> Result<Self>；request(&self, instructions: &str, input: &[Value], tools: &[Value]) -> Value；respond(&self, body: Value) -> Result<ModelReply>。ToolCall、ModelReply、Config、Limits 从本任务开始定义。

- [x] Step 1：手写一条三轮消息序列，标出 call_id；把状态地图和本机前置条件写入 00。前置条件为 Rust 1.94.0、Cargo、Bash，以及后续真实运行时所需 API Key/模型。
- [x] Step 2：在已初始化的 Git 仓库中创建根目录 Cargo 项目，固定五个依赖版本及 features；创建仅供测试的本地 HTTP stub 和六个原生响应 fixture。fake Key 固定为 test-key，模型固定为 test-model。
- [x] Step 3：写 protocol 测试，覆盖 stateless request、两个调用 id、完整 reasoning/phase、incomplete、响应内 id 缺失/重复、HTTP 500、commentary-only。核心断言为：

```rust
assert_eq!(body["store"], false);
assert!(body.get("previous_response_id").is_none());
assert_eq!(reply.calls[0].call_id, "call_1"); // fixture 的 item.id 应故意不同
assert_eq!(reply.output[0]["encrypted_content"], "opaque");
assert!(!commentary.is_final);
assert!(incomplete_result.is_err());
```

- [x] Step 4：运行 cargo test --test protocol，确认失败来自尚未实现的协议行为。
- [x] Step 5：实现上述接口和一个只打印模型文本/调用摘要的 CLI；非 completed 拒绝执行路径，保留原生 output 中的可重放字段；include 请求 reasoning.encrypted_content，不解析其内容。此时仍不执行工具。
- [x] Step 6：运行 cargo test --locked --test protocol 和 cargo check --locked；完成 01 的请求/响应观察练习。
- [x] Step 7：本地提交 feat: add native Responses protocol。这里的通过只验证本地契约，真实模型接入留到 Task 5。

## Task 2: 三个文件工具，站点 02

**Files:** Create src/tools.rs、tests/tools.rs、tutorial/v1/02-file-tools.md、docs/devlog/02-file-tools.md；Modify src/lib.rs（导出 tools）。

**Interfaces:** Consumes ToolCall、Limits；Produces schemas() -> Vec<Value>、async execute(call: &ToolCall, workspace: &Path, limits: &Limits) -> ToolResult、ToolResult::as_input(&self) -> Value、clip_utf8(text: &str, max_bytes: usize) -> (String, bool)。前三个工具先实现；bash schema 在 Task 3 补齐。

- [x] Step 1：写测试 read_write_roundtrip、edit_unique_match、edit_zero_or_multiple_matches、edit_rejects_empty_old_text、edit_rejects_oversized_result、invalid_args_return_tool_error、relative_paths_only、oversized_file、unicode_truncation。失败编辑必须同时检查结果和磁盘：

```rust
assert_eq!(result.output["ok"], false);
assert_eq!(std::fs::read_to_string(&path)?, before);
assert_eq!(result.as_input()["call_id"], "call_edit");
assert_eq!(result.as_input()["type"], "function_call_output");
```

- [x] Step 2：运行 cargo test --test tools，确认预期失败。
- [x] Step 3：实现三个 schemas 与执行分派，限制内容读取/写入，唯一精确替换，参数与 I/O 错误转成关联结果。输出字段使用 ok、content/error、truncated；as_input 的 output 为 JSON 序列化字符串。
- [x] Step 4：运行 cargo test --locked --test tools；亲手把 old_text 写错并观察文件不变，记录模型下一轮需要得到什么反馈。
- [x] Step 5：本地提交 feat: add three file tools。

## Task 3: Bash 执行与有限输出，站点 03

**Files:** Modify src/tools.rs、tests/tools.rs；Create tutorial/v1/03-bash.md、docs/devlog/03-bash.md。

**Interfaces:** 延续 execute；schemas() 返回全部四个工具。Bash 结果固定包含 ok、exit_code（整数或 null）、stdout、stderr、timed_out、truncated。

- [x] Step 1：写 bash_reports_exit_and_stderr（命令退出 7）、bash_respects_workspace、bash_bounds_both_pipes、bash_timeout_reaps_direct_child（测试 Limits 为 100ms，命令为 echo $$ > timeout.pid; exec sleep 10；返回后读取 PID，kill -0 应失败）。检查：

```rust
assert_eq!(nonzero.output["exit_code"], 7);
assert_eq!(timeout.output["timed_out"], true);
assert_eq!(large.output["truncated"], true);
assert!(stdout.len() <= 4096 && stderr.len() <= 4096); // 使用 ASCII 输出 fixture
```

- [x] Step 2：运行 cargo test --test tools bash_，确认预期失败。
- [x] Step 3：以 bash -c 和 current_dir 实现命令执行；stdin 关闭，双管道并发有限采集；timeout 覆盖等待和采集，发生时显式 kill/reap 直接子进程，终止采集等待。不要以 wait_with_output 后截断代替采集上限。
- [x] Step 4：运行 cargo test --locked --test tools；超时实验记录 PID/存活情况，说明直接子进程验证和后代进程限制。
- [x] Step 5：本地提交 feat: add bounded Bash execution。

## Task 4: 闭环、消息关联与停止，站点 04

**Files:** Modify src/lib.rs、src/main.rs；Create tests/loop.rs、tutorial/v1/04-agent-loop.md、docs/devlog/04-agent-loop.md。

**Interfaces:** Consumes ModelClient、schemas、execute、as_input；Produces async run(config: Config) -> Result<RunReport>，StopReason 与 RunReport 按共享类型定义。CLI 正常 Final 退出 0，上限退出 2，配置/协议/API 错误退出 1；退出 0 不宣称测试已通过。

- [x] Step 1：写 two_calls_then_final、tool_error_then_corrected_call、incomplete_never_mutates、commentary_then_final、max_turns_stops、request_budget_stops。使用 stub requests 验证第二次请求带上完整 output、对应结果和 reasoning/phase；主调用顺序固定。
- [x] Step 2：运行 cargo test --test loop，确认预期失败。
- [x] Step 3：实现“构造请求 → 检查序列化大小 → 请求 → 完整响应校验 → 串行执行 → 追加结果 → 继续”。只有 is_final 且无调用才结束；未知工具/坏参数形成反馈而不是 panic。
- [x] Step 4：运行 cargo test --locked --test loop；核对调用预算：

```rust
assert_eq!(report.reason, StopReason::Final);
assert_eq!(requests[1]["input"][result_index]["call_id"], "call_2");
assert_eq!(limited.requests().len(), 2); // 测试 max_turns 为 2
assert_eq!(std::fs::read_to_string(&path)?, before); // incomplete 场景
```

- [x] Step 5：终端显示 turn、工具名、结果状态、退出码和结束原因；不显示 API Key。完成 04 的手绘闭环复盘并本地提交 feat: complete the coding loop。

## Task 5: v1 真正修复任务与冻结，站点 05

**Files:** Create fixtures/buggy-rust/{lib.rs,tests.rs,README.md}、fixtures/verifier/verify.rs、scripts/{verify-playground.sh,count-lines.py}、tutorial/v1/05-v1-coding-task.md、docs/devlog/05-v1-coding-task.md；Modify README.md、.gitignore、tests/loop.rs；Create evidence/v1/ 的真实运行材料。

**Interfaces:** 练习代码公开函数为 pub fn last_index(len: usize) -> Option<usize>。初始实现故意把非空长度作为索引；空输入也错误。工作区测试覆盖 len 0、1、10；独立 verifier 位于 harness 内，使用 CRABLOOP_SOLUTION 传入待检 lib.rs 的绝对路径，断言 None、Some(0)、Some(9)。计数脚本扫描全部 src/**/*.rs。

- [x] Step 1：创建练习 fixture，运行其测试，确认原始项目失败；检查独立 verifier 同样失败。不要把失败 fixture 纳入 harness 自身的通过测试集。
- [x] Step 2：写 scripted_bugfix_loop 测试，由 fake Responses 驱动“read_file → bash 原测试失败 → write_file 创建 notes.md → edit_file 修复 → bash 测试通过 → 最终回答”，在真实临时文件/进程中覆盖四工具；运行 cargo test --test loop scripted_bugfix_loop。
- [x] Step 3：复制 fixture 到可丢弃 playground/bugfix；设置自己的 OPENAI_API_KEY 和 CRABLOOP_MODEL，以命令 cargo run --locked -- --workspace playground/bugfix "修复 last_index，先运行测试；保留函数签名和测试文件，完成后再次验证" 进行首次真实 Responses 运行。
- [x] Step 4：从 harness 根目录执行 bash scripts/verify-playground.sh playground/bugfix，并检查源码差异、原测试和独立 verifier。若 provider/模型失败，保存失败证据，修正问题后再验收；不把 stub 成功记成真实成功。
- [x] Step 5：保存终端轨迹、模型 ID、日期、差异、两套测试结果和复盘到 evidence/v1/；凭实际证据填写 v1 验收表。
- [x] Step 6：执行 cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings、cargo test --locked、python3 scripts/count-lines.py；报告真实行数，若明显超目标先删除范围外功能，保留错误处理。
- [x] Step 7：本地提交 milestone: finish v1，再创建 v1 tag。此后仅在该版本上逐步增加 v2。

## v1 收束后的接口调整

Task 5 收束时取消了只包装配置的 ModelClient 和只包装回传的 ToolResult。HTTP 连接池由 run 持有，配置保持单一来源；工具只返回业务 JSON，Responses 的 call_id 关联与 function_call_output 构造属于 loop。

当前接口为 model::request(config: &Config, instructions: &str, input: &[Value], tools: &[Value]) -> Value；async model::respond(client: &reqwest::Client, config: &Config, body: Value) -> Result<ModelReply>；async tools::execute(call: &ToolCall, workspace: &Path, limits: &Limits) -> Value。早期章节的接口保留在相应 step tag 中。

## Task 6: 固定上下文、记录与整组投影，站点 06

**Files:** Create src/context.rs、src/journal.rs、tests/context.rs、tutorial/v2/06-context-projection.md、docs/devlog/06-context-projection.md；Modify src/lib.rs、src/main.rs、README.md。

**Interfaces:** ContextPolicy { hard_bytes: usize, soft_bytes: usize, summary_bytes: usize, keep_rounds: usize }；ContextPolicy::new(hard_bytes: usize) -> Result<Self> 按全局规则派生值。ContextState { instructions: String, task: String, summary: Option<String>, rounds: Vec<Vec<Value>> }；new(instructions: String, task: String) -> Self、append_round(&mut self, round: Vec<Value>) -> Result<()>、input(&self) -> Vec<Value>、request_bytes(config: &Config, instructions: &str, input: &[Value], tools: &[Value]) -> usize。Journal::create(path: &Path) -> io::Result<Self>、record(&mut self, kind: &str, data: &Value) -> io::Result<()>。

- [ ] Step 1：写 complete_round_preserves_all_items、rejects_orphan_result、fixed_task_and_agents_survive_projection、request_bytes_includes_tools_and_ciphertext、journal_is_append_only、invalid_budget_rejected、invalid_agents_rejected。覆盖缺失 AGENTS 跳过、存在但非 UTF-8/超限失败、hard 小于 4,096 失败。
- [ ] Step 2：运行 cargo test --test context，确认预期失败。
- [ ] Step 3：实现纯投影与 byte 预算；根目录 AGENTS.md 加入每次 instructions，任务原文固定保留。先仅支持预算不足时停止，尚不实现摘要。
- [ ] Step 4：实现 JSONL，每次请求、响应、工具意图/结果及停止都追加并 flush；日志新建于 .crabloop/run-TIMESTAMP-PID.jsonl，创建失败就结束。记录可包括 usage，但不得包含 Authorization。
- [ ] Step 5：运行 cargo test --locked --test context 与原 v1 回归；检查 log 事件数量不因输入裁剪减少，原任务与指令未变化：

```rust
assert_eq!(state.input()[0]["content"], original_task);
assert_eq!(after_journal_events, before_journal_events);
assert!(projected_request_bytes < full_request_bytes);
```

- [ ] Step 6：做一次只保留最近完整组的离线投影实验，故意删掉早期测试信息，记录损失；本地提交 feat: separate history records from model context。

## Task 7: 预算触发的手写摘要，站点 07

**Files:** Modify src/context.rs、src/lib.rs；Create tests/compaction.rs、tutorial/v2/07-compaction.md、docs/devlog/07-compaction.md。

**Interfaces:** async ContextState::prepare(&mut self, client: &reqwest::Client, config: &Config, tools: &[Value], policy: &ContextPolicy, journal: &mut Journal) -> Result<Vec<Value>>；返回可直接用于下一次主请求的 input。摘要调用继续使用 model::request/respond，但 tools 为空；Run 内记录主调用数与摘要调用数。

- [ ] Step 1：写 below_threshold_skips_summary、compaction_preserves_recent_two_rounds、summary_request_has_own_budget、oversized_tail_stops、empty_or_incomplete_summary_is_rejected、failed_summary_does_not_commit、summary_does_not_execute_tools。stub 的有效摘要使用已完成、修改文件、验证结果、剩余工作、不确定事项五段。
- [ ] Step 2：运行 cargo test --test compaction，确认预期失败。
- [ ] Step 3：按设计说明六步实现 prepare；总结较早前缀和上一份摘要，原任务/AGENTS 不交给摘要替代，先验证新输入再提交。密文 reasoning 保持不透明，随完整组处理。
- [ ] Step 4：运行 cargo test --locked --test compaction；验证失败的原状态和成功候选：

```rust
assert_eq!(failed_state.input(), before_input);
assert_eq!(compacted.rounds.len(), 2);
assert!(compacted_request_bytes < policy.soft_bytes);
assert_eq!(summary_request["tools"], serde_json::json!([]));
```

- [ ] Step 5：主 loop 在每次主请求前调用 prepare；一轮最多摘要一次，累计最多 15 次，日志区分 main/summary usage、压缩前后 bytes 和失败原因。原请求预算保护继续保留。
- [ ] Step 6：回归 cargo test --locked，完成强制小预算实验并本地提交 feat: add bounded context compaction。

## Task 8: 协议与事实保留对照，站点 08

**Files:** Modify tests/context.rs、tests/compaction.rs；Create tests/fixtures/context/{history,faithful_summary,misleading_summary}.json、tutorial/v2/08-context-experiments.md、docs/devlog/08-context-experiments.md、evidence/v2/ 对照材料。

**Interfaces:** 对同一份 history 构造完整输入、最近两组输入、摘要+最近两组输入；利用已有 request_bytes、ContextState 和 fake Responses。五段摘要的结构检查由程序完成；语义核验由 fixture 断言和真实运行人工复核分别承担。

- [ ] Step 1：构造含早期测试失败、未完成修改、最新两个调用组的固定 history；把 faithful_summary 与故意写成“测试通过”的 misleading_summary 分别送入实验。
- [ ] Step 2：写 context_comparison_preserves_protocol 与 misleading_summary_fails_semantic_verifier；独立断言早期测试仍为 failed、未运行测试仍为 not_run、剩余任务仍 pending。预期：误导摘要可满足五段结构，却必须被事实核验判为失败。
- [ ] Step 3：运行 cargo test --locked --test context --test compaction；保存三方案的输入 bytes、调用/结果关联检查和事实核验表，不把 fake summary 结果当成模型压缩质量。
- [ ] Step 4：真实模型实验先以 CRABLOOP_CONTEXT_BYTES=8,192 起步，测量固定输入、最近两组和摘要预留的大小；必要时按 16,384 → 32,768 → 49,152 校准，选择候选输入装得下且完整历史能触发整理的预算。使用多轮任务触发至少一次压缩，记录摘要请求、前后输入、usage、最终差异和测试结果。若仍装不下，记录明确停止，不删除半组“凑成功”。
- [ ] Step 5：复核五类信息：任务约束、修改文件、测试结果、剩余工作、不确定事项。对照至少一次无压缩运行；费用/时延只记录实测，不预设压缩更便宜。
- [ ] Step 6：把一次损失案例与修正建议写入 08，保留原始证据；本地提交 test: verify context projection and compaction behavior。

## Task 9: v2 综合验收与重建练习，站点 09

**Files:** Modify README.md、scripts/count-lines.py（仅确有遗漏时）；Create tutorial/v2/09-v2-retrospective.md、docs/devlog/09-v2-retrospective.md、evidence/v2/acceptance.md。

**Interfaces:** 不增加运行时能力；沿用 v1 的外部 verifier、v2 journal 和 context 比较方法。

- [ ] Step 1：在新的 playground 副本中运行强制压缩修复任务；确认日志至少一条成功压缩，原任务/AGENTS 固定保留，独立 verifier 通过；单独记录未通过或模型误报完成的情况。
- [ ] Step 2：运行 cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings、cargo test --locked、python3 scripts/count-lines.py，以及对应 playground 的外部验收。
- [ ] Step 3：填写 evidence/v2/acceptance.md：实际源代码行数、协议测试、Bash/文件真实测试、压缩次数与输入大小、真实模型 ID/日期、外部验收和已知局限。记录层、运行机制与模型任务结果分别给结论。
- [ ] Step 4：完成重建练习：不看参考代码，在单独 scratch 目录重写 v1 loop；沿一个 v2 日志解释历史在哪里、每轮输入是什么、失败怎样改变动作、压缩丢失了什么。该练习不修改冻结版本。
- [ ] Step 5：本地提交 milestone: finish v2，并创建 v2 tag。后续选修一次只选一个：原生 compaction 对照、Streaming/取消、会话恢复、进程树治理、MCP 或只读 subagent。

## 两个版本的完成门槛

| 检查 | v1 | v2 |
|---|---|---|
| 能从入口读懂完整核心 | 约 300 行，四工具与明确结束原因 | 约 500 行，清楚分辨记录/投影/压缩 |
| 确定性证据 | 原生协议、文件/Bash、loop 故障测试 | 加上下文分组、预算、压缩失败与事实对照 |
| 真实模型证据 | 新副本修复任务；外部 verifier 通过 | 至少一次成功压缩后继续任务；外部 verifier 通过 |
| 学习者解释 | 模型看到了什么，为什么继续 | 当前输入为何这样选择，哪些信息被整理或丢失 |
| 版本保留 | v1 tag，差异与真实运行材料 | v2 tag，压缩对照与真实运行材料 |

本计划已获执行授权。助手顺序完成各阶段的参考实现和教学材料；学习者随后可按对应提交逐章亲手重建。实施中持续更新 AGENTS.md，局部设计修正写入开发记录，并保持上述版本边界与证据标准。
