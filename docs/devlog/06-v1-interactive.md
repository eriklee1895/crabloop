# v1.1：提前补上可用的交互入口

日期：2026-10-12。起点：原 `v1` 标签（`a570154`）。本分支为独立的早期学习快照，原 `v1`/`v2` 引用不移动，也不把这个较早的 backend 合并覆盖 main。

用户明确要求从早期版本就能直接启动、多轮对话，并用 `-p` 执行单次任务。原来的 `--workspace DIR TASK` 强制参数不符合这个使用方式，因此回移主线已实现的 CLI 和 rustyline 终端界面，把 v1 的 loop 变为持有完整内存历史的 `Session`。

- `src/lib.rs`：`Session` 保存 client、工具 schema 和 `Vec<Value>`；新 user message、全部 Responses output（包括最终回答）以及工具结果依次追加。`run(Config)` 继续作为单次包装。
- `src/cli.rs` / `src/terminal.rs`：无参数 TTY 聊天、`-p/--print`、兼容位置参数和 stdin、`/new`、`/help`、`/exit`、输入编辑及进度。`.env` 作为数据读取，进程值优先；provider 与 key 配对。
- 协议/传输错误与中断后停止使用当前 Session。需要显式 `/new`，不自动重放工具。v1 的直接子进程取消边界保留，未增加进程树管理。
- `src/model.rs` / `src/tools.rs` 展开协议 JSON 的阅读格式；四个工具和协议行为沿用 v1。没有引入压缩、journal 或 root AGENTS.md 自动加载。

先回移 CLI 和会话测试，观察到旧二进制拒绝 11 种新入口/配置路径，`Session` 接口也不存在。再实现状态持有者和入口，相关测试转绿。指定工作区的 CLI 测试直接检查 `work/note.txt` 的内容，避免依赖 v2 才有的日志目录。

验证命令：

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 -m unittest discover -s tests -p '*_test.py'
python3 scripts/count-lines.py
```

结果：43 项 Rust 测试、3 项 Python 验收器测试通过，fmt/clippy 通过。测试覆盖真实本地文件/Bash 执行与本机 fake Responses，不调用真实 provider。原 v1 的真实模型任务记录继续放在 `evidence/v1/`，它们不作为 v1.1 新增交互的实测证据。

另外在真实 PTY 中用测试配置直接启动二进制，不提供任务参数；观察到 `crab>` 提示符，输入 `/exit` 后以 0 退出。该检查没有提交模型任务。

统计：原 v1 为 314 代码行 / 319 物理行；v1.1 为 **571 / 581**，包含全部生产 Rust。新增交互范围使完整代码超过最初约 300 行目标，保留并分别报告这两个学习状态。没有通过移动实现或压缩格式降低计数。

仍保留 v1 的限制：完整内存历史在预算不足时停止，退出即丢失；非流式回答；无 OS sandbox；取消不保证清理所有后代进程；同一响应含 commentary 和 final 文本时最终展示仍会合并两段。后者不影响完整 output 的历史回传。
