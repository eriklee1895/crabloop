# 从空目录走到 v1：六个只做加法的代码增量

正文解释代码为什么这样写；这里的补丁补齐每一步需要的导入、类型、函数和测试。第 01–06 章各对应一个补丁，按顺序应用后，得到与 `milestone/v1` 相同的全部 `src` 文件。

每个补丁只在前一步的基础上**增加**代码：前面学过的函数签名和类型不会在后面被换掉。你在第 01 章认识的 `model::request`，到第 06 章仍然是同一个函数。

## 先准备两个位置

在已经检出 `milestone/v1` 的参考仓库根目录执行：

```bash
export CRABLOOP_SOURCE="$PWD"
mkdir -p playground/v1-lesson
cd playground/v1-lesson
git init -q
cp "$CRABLOOP_SOURCE/Cargo.lock" .
```

`CRABLOOP_SOURCE` 指向参考仓库，它只提供补丁和对照；之后编辑代码、运行 `cargo`、应用补丁，都在练习目录 `playground/v1-lesson` 中进行。`playground/` 已被参考仓库的 `.gitignore` 忽略。

复制 `Cargo.lock` 是为了让依赖解析与参考仓库完全一致，后面的命令都带 `--locked`。如果 `v1-lesson` 已经存在，换一个目录名即可，保留之前的练习成果。

## 每一步增加什么

| 补丁 | 章节 | 这一步加入的能力 |
|---|---|---|
| [01-responses.patch](01-responses.patch) | 01 | Cargo 工程、`Config::from_env`、`model::request`/`respond`、`examples/probe.rs` |
| [02-file-tools.patch](02-file-tools.patch) | 02 | `tools::schemas`、`execute`，以及 `read_file`、`write_file`、`edit_file` |
| [03-bash.patch](03-bash.patch) | 03 | `bash` 工具：有界输出、超时与子进程回收 |
| [04-agent-loop.patch](04-agent-loop.patch) | 04 | `Stop` 与 `run_turn`：请求、执行、回传、继续或停止 |
| [05-coding-task.patch](05-coding-task.patch) | 05 | `src/main.rs` 入口、练习材料与独立验收脚本 |
| [06-conversation.patch](06-conversation.patch) | 06 | 只加测试：检查第二个任务携带第一个任务的历史 |

每章都会给出自己的应用命令，形式相同：

```bash
git apply --check "$CRABLOOP_SOURCE/tutorial/v1/patches/01-responses.patch"
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/01-responses.patch"
cargo test --locked
```

补丁依赖前一步的状态，不能跳着应用。如果你按正文手写了代码，与补丁上下文有出入，可以打开补丁对照 diff，逐文件补上差异，不必为了通过检查覆盖自己的练习。

## 核对整条路线

在参考仓库根目录执行：

```bash
cargo fetch --locked
python3 scripts/check-lessons.py
```

脚本在临时目录里从空仓库开始重放六个补丁，每一步都检查 `cargo fmt --check` 和 `cargo test`，最后确认补丁生成的每个文件都与参考仓库一致。测试使用本地 HTTP 服务、临时文件和真实进程，不需要模型密钥。

修改教学补丁的贡献者，可以用 `python3 scripts/check-lessons.py --v1-ref HEAD` 对照某个已提交的版本。
