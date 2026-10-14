# main README 精简

日期：2026-10-14。

沿用用户对 v1 README 的要求，main 首页集中说明项目定位、Quick Start、学习版本选择、核心源码与常用文档。保留 Banner、徽章与交互动图，删除重复的行数统计、完整未来版本表、provider 决策表、预算参数、验收流水和开发历史。

核对 `src/cli.rs`、`src/terminal.rs` 和 `Session::turn`，保留 main 的聊天与 `-p` 用法，区分 v1 单任务、v2 上下文学习和 main 完整实现。JSONL 观察记录与尚未实现的跨进程恢复保持区分。

AGENTS.md 记录首页精简约定，并删除残留的 v1 `--max 399` 指令，保持文件低于运行时的 8 KiB 上限。

验证：README 本地链接、页内锚点、代码块和 Bash 示例语法、`git diff --check`、fmt、clippy、65 项 Rust 与 6 项 Python 测试通过。测试使用独立 target 目录并禁用编译包装器，避免复用带旧临时目录路径的缓存。本次没有修改 Rust 实现或进行新的真实模型验收。
