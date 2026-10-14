# CrabLoop 🦀

用 Rust 从零写一个能读文件、修改代码、运行测试，再根据真实反馈继续行动的 mini agent。

这是 **milestone/v1 的最小教学实现**：388 代码行 / 449 物理行，统计包含全部 src/**/*.rs。目标约 300 行、硬上限 <400 代码行；保持正常 rustfmt，没有把运行逻辑移出 src。v1 学习统一使用这一份代码：src/cli.rs 为 62 代码行，其余实现合计 326 行。历史标签仅保存步骤与验收来源。

## 先运行一个任务

需要 Rust 1.94.0 和 Bash。在当前目录的 .env 配置 DEEPSEEK_API_KEY，然后：

```bash
cargo run
```

看到提示后输入一个任务，按 Enter 提交；完成后程序退出。脚本和多行任务也从 stdin 传入：

```bash
printf '%s\n' "检查当前目录" | cargo run --quiet
```

v1 不接受命令行参数，包括任务文本、`-p/--print` 和 `-h/--help`。误传参数会直接报错并指向本 README；空输入或 EOF 直接退出，不读取凭证、不请求模型。启动和配置说明集中在这里及 [.env.example](.env.example)。专业 CLI Help 的排版、高亮与颜色属于[后续独立迭代](docs/cli-help-plan.md)。

工作区就是启动目录。要操作另一个目录，先构建，再从那个目录调用二进制：

```bash
cargo build --locked
crabloop_bin="$PWD/target/debug/crabloop"
cd /path/to/project
printf '%s\n' "读取项目说明并检查测试" | "$crabloop_bin"
```

配置从启动目录的 .env 和进程环境读取，进程环境优先。默认 DeepSeek；使用 OpenAI 或兼容端点时显式设置 CRABLOOP_PROVIDER=openai，再提供 OPENAI_API_KEY、OPENAI_BASE_URL 和 CRABLOOP_MODEL。完整端点可由 CRABLOOP_API_URL 覆盖。配置格式见 [.env.example](.env.example)，凭证不可提交。

## 从哪里读核心

从 [src/lib.rs](src/lib.rs) 的 run 函数开始。input 保存当前任务的完整历史：请求模型、追加 response.output、执行工具、用原 call_id 追加结果，再进入下一轮。模型给出最终回答或耗尽预算时停止。

| 文件 | 职责 |
|---|---|
| src/main.rs | 调用入口、显示最终答案和退出状态 |
| src/cli.rs | 单次任务输入和最小配置 |
| src/lib.rs | 限制、停止原因与手写 agent loop |
| src/model.rs | Responses 请求和完整响应校验 |
| src/tools.rs | read_file、write_file、edit_file、bash |

一个用户任务可以触发多轮模型和工具交互。工具失败是下一次模型输入的一部分；模型说完成后，仍需独立检查文件和测试结果。v1 不包含多轮用户聊天、输入编辑器、SSE、摘要或会话恢复；这些逐步在后续实现中展开。日常多轮界面仍可在 main 使用。

v1 的定稿提交为 `83a4e83`；日常阅读统一使用 `milestone/v1`。[GA 审查与验收](evidence/v1/ga-review.md)记录本次格式、结构、测试和真实任务结果。

## 阅读与验证

从[教程入口](tutorial/README.md)进入 00–05 的最小循环学习；[七个增量](tutorial/v1/patches/README.md)从空目录重建后，全部 src 文件与当前分支一致。阅读和动手都以同一份 v1 为终点。已有验收和开发记录保留原来的含义；当前入口通过 42 项 Rust、5 项 Python 离线回归；此前的真实 DeepSeek 修复和独立验证保留为历史证据，本次入口删减未重跑真实模型。范围见[教学边界](docs/v1-scope.md)，本次入口删减见[实施记录](docs/devlog/26-v1-stdin-cli.md)。

```bash
python3 scripts/count-lines.py --max 399
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

行数门禁超标会返回非零退出码。文件大小、工具输出、请求预算和 Bash 超时仍有限制；v1 不提供 OS sandbox、原子写入回滚或完整进程树清理。模型调用失败明确结束，不自动重放工具。
