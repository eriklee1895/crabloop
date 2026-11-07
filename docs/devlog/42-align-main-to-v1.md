# main 与 milestone/v1 对齐

日期：2026-11-07。Erik 批准把 main 的代码对齐到 v1 GA，并暂时保留旧 main 历史。有用的旧实现留到 v2，按小步迁回并重新验证；这次不恢复那些代码。

## 合并

合并提交的第一父提交是旧 main `e06101f`，第二父提交是 milestone/v1 `e1baa80`（GA 运行基线 `9a8077e`）。该提交的树等于 milestone/v1，树对象为 `0a486629`。因此合入 main 之后，milestone/v1 成为 main 的祖先，main 可以快进，不需要 force push，也不改写已有提交。

标签没有移动或删除。`v1` 仍指向 `a570154`，`v2` 与 `step-06-context` 至 `step-interactive-chat` 仍指向旧 main 上的历史实现。没有推送 main 或 `milestone/*`。

这次跟进只改文档。相对 milestone/v1，`src/`、测试、脚本和 Cargo 没有差异，所以不再做真实模型运行。GA 证据仍是 [当前复核](../../evidence/v1/ga-current.md)。

## v2 规模

v2 从约 500 行改为约 1000 行。这是教学参考，不是硬上限，也不加入统计行数并失败的测试或脚本。

v2 包含 context state、projection、compaction 和验收；reasoning effort 只在明确设置时发送，未设置时省略该字段；同时显示 token 预算/用量，并把工具状态写清楚。工具安全边界（bash 的 clean env、审批、工作区隔离）和 TOML/`.env` 配置仍属于 v3。TUI 放在 v2 之后。

[design.md](../design.md) 和 [plan.md](../plan.md) 仍保留当初约 500 行的学习计划。当前预算以本记录、[roadmap](../roadmap.md) 和 [v1 scope](../v1-scope.md) 为准。

## 旧 main 里值得迁回的部分

旧实现留在第一父提交和 `v2` 标签 `5a5e96e` 上。迁回时沿用 v1 的类型和协议校验，不整包拷贝旧 CLI。

- **Context state 与 projection。** `5926257` 的 `src/context.rs` 把任务、固定指令、摘要和完整 round 分开，`input()` 再拼出模型输入。`tests/context.rs` 检查完整组、孤立或重复 `call_id`，以及请求字节包含指令、工具和密文。对应 v2 的 state 和 projection。注意：`append_round` 只检查调用是否配平，没有 v1 的 `call_id.trim()`，也不走 `model.rs` 的 serde 类型。迁回时用 v1 的校验标准。
- **Compaction。** `2cb5990` 与 `a5b14ad` 的 `ContextPolicy` 和 `prepare` 先做摘要候选，确认正文、篇幅和压缩后的输入，再替换状态。失败、超长或 HTTP 错误都保留原历史。`tests/compaction.rs` 覆盖阈值、最近两组、摘要不计入主轮次，以及“标签齐全但仍误导”的摘要。对应 compaction。形状检查不能代替独立事实验收。摘要请求把 `reasoning.effort` 固定写成 `none`；v2 应只在明确选择时发送该字段，普通请求继续省略。摘要模型是否接受 `none` 要在迁回时重新核对。
- **运行记录。** `src/journal.rs` 把请求、响应、工具和停止写成工作区里的 JSONL，并与模型输入分开。对应 v2 的执行记录。日志含完整请求体，测试确认其中没有 API key。这还不是 v5 的可恢复 session，不要把 `.crabloop/run-*.jsonl` 提交进仓库。
- **预算与 usage。** 旧代码用序列化请求体的字节数做预算，并把响应里的 `usage` 留在日志中，终端没有单独的 token 行。对应 token 预算/用量显示。字节数不是 tokenizer，也不要把它说成供应商 token。v1 的输出上限是 131,072；旧 main 的默认值是 32,768，不要把这个更小的上限带回来。
- **工具状态。** v1 已经在 stderr 打印 `→ name args · ok=`。`4214a38` 的 `src/terminal.rs` 把路径或命令收成一行，并用成功/失败区分结果，commentary 出现在工具之前。对应更清楚的工具状态。只借鉴这种短状态；不要带回 rustyline、颜色和整段工具 JSON。单次模式打印完整 JSON 比 v1 更吵。

可以当提纲、需要按 v1 接口重写的材料：`tutorial/v2/06` 至 `09`，以及旧 main 的 `docs/devlog/06-context-projection.md` 至 `09-v2-retrospective.md`。旧 `evidence/v2/` 只说明当时那次运行，不能充当新 v2 的验收。

## 不迁回 v2 的部分

- `src/cli.rs` 的 `.env`、provider 选择、`--workspace`、`-p` 和 `--chat`。配置留到 v3。
- `src/terminal.rs` 整个交互层。v1 已有 stdin 连续任务；TUI 在 v2 之后。
- 旧 `src/model.rs` 的手写字段解析。它不拒绝空白 `call_id`，畸形 output item 会跳过而不是失败。v1 在 `d732663` 用 serde 类型、`call_id.trim()` 和失败即拒绝保住了这些检查。可以保留 `usage` 和可替换的 instructions/tools，但解析必须回到 v1 的标准。
- 旧 `src/tools.rs` 的 bash。它去掉了 `OPENAI_API_KEY` 的 `env_remove`。v1 的行为更严格；clean env、审批和隔离仍是 v3。
- 更长的 CrabLoop 人设文案、吉祥物、banner，以及 `scripts/render-mascot.py`、`scripts/record-terminal.py`。v2 要的是把项目 `AGENTS.md` 固定在摘要之外，不是替换 v1 的短指令。
- `tests/live_config_test.py` 和旧 main 上替换过的 `tests/protocol.rs`。协议测试以 v1 这套为准。

对齐会从结果树中拿掉旧 main 的 `.github/workflows/ci.yml`。那份工作流只在当时的 main 上运行 fmt、clippy、Rust 测试和 Python unittest。v1 没有这份文件，所以合入之后 GitHub 不再自动跑这些检查，直到单独把 CI 加回来。CI 不是 v2 功能。

## 验证

在文档改动之外，源码与 milestone/v1 相同，因此没有真实模型运行。本机门禁全部通过：

- `cargo fmt --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked`：33 项 Rust 测试通过（cli 3、loop 9、protocol 8、tools 13）
- `python3 -m unittest discover -s tests -p '*_test.py'`：6 项通过
- `python3 scripts/check-lessons.py`：六个教学阶段回放通过，最终统计仍是 324 代码行 / 370 物理行

旧 main 上的 GitHub Actions 只监听 main 的 push 和 pull request，工作流文件不在 v1 树里。本 PR 合入后，main 上不再有这份 CI，直到以后单独加回。
