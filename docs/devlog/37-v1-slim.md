# v1 收敛到约 300 行

2026-10-17。Erik 审阅只读评审（436 代码行 / 495 物理行）后批准精简方案：v1 只演示请求模型 → 执行工具 → 回传结果 → 继续或停止，规模上限 400 行，约 300–350 行为宜；v2 目标 500–600 行，v3 1000 行以上。

## 删去或后移

- `src/cli.rs` 整个删除：参数拒绝、启动信息、终端/管道两套读取、stdin 字节上限。
- `.env` 读取与 dotenvy 依赖、`CRABLOOP_PROVIDER` 的 DeepSeek/OpenAI 分支、`CRABLOOP_API_URL`，移到 v3 配置。
- `run(Config)` 包装、`RunReport`、`Limits`（8 个字段），以及重复 call_id 检查。
- `run_turn` 中约 20 行工具状态展示，收成一行 `→ 工具名 参数 · ok=...`。
- PTY 聊天测试 `tests/chat_test.py`、`.env.example`。

## 改成什么

- 配置：`OPENAI_API_KEY`、`OPENAI_BASE_URL`（默认 OpenAI）、`CRABLOOP_MODEL`，与第 01 章探针一致；DeepSeek 只需设置 base URL 和模型名。
- 入口：stdin 每个非空行是一个任务，终端与管道同一路径，`main` 持有历史。非 Final 停止返回 `ExitCode` 2，不再 `process::exit`。
- 类型：`enum Stop { Final(String), MaxTurns, ContextLimit }`；`Reply.final_text: Option<String>`；外层 `Response` 与 `Item`（`FunctionCall`/`Message`/`Other`）用 serde 解析，`output: Vec<Value>` 原样保留回放。
- 工具：`match` 分派加 `resolve`/`read`/`write` 小函数；`clip` 用 `floor_char_boundary` 并返回借用；`drain` 改为 `take` + `read_to_end` + `copy` 到 `sink`；超时分支只在进程仍存活时 kill。
- 安全：bash 子进程 `env_remove("OPENAI_API_KEY")`。旧版只有放在 `.env` 里的密钥不进入子进程，export 的密钥会被继承。
- 新增 `examples/probe.rs`：第 01 章用最终的 `model::request`/`respond` 发一次真实请求。

## 教程与补丁

旧路线的问题是 01–04 章使用阶段接口（`ModelClient`、`ToolResult::as_input`、`--workspace` 参数），第 05 章用两个合计约 2,800 行的补丁一次换成最终代码。新路线改为六个补丁，与第 01–06 章一一对应，每个补丁只增加代码：第 01 章写下的签名到最后不变。`Cargo.lock` 不再放进补丁，读者从参考仓库复制。第 06 章只加测试，并用“把 `let mut input` 移进循环”的实验观察历史寿命。

`scripts/check-lessons.py` 改为默认对照工作区：每一步检查 `cargo fmt --check` 与 `cargo test`，最后要求补丁生成的每个文件都与参考仓库一致。补丁由仓库外的生成脚本从最终文件按阶段切片得到，保证“只做加法”。

## 验证

- `cargo fmt --check`、`cargo clippy --locked --all-targets -- -D warnings`、`cargo test --locked`：29 个 Rust 测试通过（cli 3 / loop 8 / protocol 5 / tools 13）。
- Python：`live_driver_test.py`、`verify_report_test.py` 共 5 个测试通过（取证时修 verifier 后为 6 个，见 38）。
- `python3 scripts/check-lessons.py`：六步全部通过，最终文件一致。
- 手动核对第 04 章“断开回传”和第 06 章“历史只活一轮”两个实验：改动后对应测试按预期失败，恢复后通过。
- 统计：304 代码行 / 350 物理行（lib 65、main 24、model 75、tools 140）。
- 当时还没跑真实模型；之后补的证据见 `38-v1-slim-live.md`。
