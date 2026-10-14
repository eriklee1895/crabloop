# 从空目录走到 v1：每一步完整的代码增量

正文带你理解代码为什么这样写，这里的补丁补齐每一步需要的导入、类型、函数和测试。顺序应用七个增量，最后得到与 `milestone/v1` 相同的全部 `src` 文件。

练习工程放在自己的目录中。参考仓库保存源码与补丁，负责提供对照，不直接承接这些增量。

## 先记住两个位置

在已经检出 `milestone/v1` 的参考仓库根目录，记住它的绝对路径，然后创建一个尚不存在的练习目录：

```bash
export CRABLOOP_SOURCE="$PWD"
mkdir -p playground
mkdir playground/v1-lesson
cd playground/v1-lesson
git init -q
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/00-probe.patch"
cargo check --locked
```

如果 `v1-lesson` 已经存在，就换一个名字，保留之前的练习成果。`CRABLOOP_SOURCE` 指向参考仓库；后续执行 `cargo`、编辑代码和应用补丁，都在练习目录。

`00-probe.patch` 包含 `Cargo.lock`，所以看起来很长。第一次阅读先看 `Cargo.toml` 和 `src/main.rs`：它们构成一次请求的小探针。此时构建无需模型密钥，真正请求服务的配置见第 01 章。

如果你已经按第 01 章手写了探针，就从 01 增量继续，不要再把 00 应用一遍。手写代码与补丁上下文有差异时，可以阅读 diff，逐文件补上相应变化。

## 每一步增加什么

| 补丁 | 对应章节 | 这一阶段加入的能力 |
|---|---|---|
| [00-probe.patch](00-probe.patch) | 01 的小探针 | 发一次 HTTP 请求，打印响应 |
| [01-responses.patch](01-responses.patch) | 01 | 配置、响应解析与协议校验 |
| [02-file-tools.patch](02-file-tools.patch) | 02 | 读取、写入和唯一匹配编辑 |
| [03-bash.patch](03-bash.patch) | 03 | 执行 Bash，收集输出与退出状态 |
| [04-agent-loop.patch](04-agent-loop.patch) | 04 | 历史、工具回传与停止条件 |
| [05-v1.patch](05-v1.patch) | 05 | 综合任务、独立验收与接口整理 |
| [06-v1-entry.patch](06-v1-entry.patch) | 05–06 | 当前 v1 的 stdin 入口、配置与最小连续对话 |

补丁编号和章节编号并不完全一一对应：第 01 章从探针走到协议解析；第 05 章应用最后两个增量，得到完整 v1；第 06 章再拆开其中的连续对话，不需要额外应用补丁。

例如，读完第 01 章以后，在练习目录运行：

```bash
git apply --check "$CRABLOOP_SOURCE/tutorial/v1/patches/01-responses.patch"
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/01-responses.patch"
cargo test --locked --test protocol
```

后面的章节给出了各自的应用和实验命令。补丁依赖前一步的状态，不能直接把 04 应用到空目录。如果已经做了自己的修改，先比较 diff 再决定怎样合并，不要覆盖练习成果来强行通过检查。

`step-*` 标签保存当时的源码，方便解释阶段接口。最终阅读和练习都以 `milestone/v1` 为终点：`main` 持有历史，`run_turn` 执行当前任务；终端能继续提问，管道输入仍只运行一次。

## 核对整条路线

在参考仓库根目录执行：

```bash
cargo fetch --locked
python3 scripts/check-lessons.py
```

检查脚本会在临时目录重放所有补丁，运行各阶段测试，并核对最后的全部 `src` 与 `milestone/v1` 一致。测试使用本地 HTTP 服务、临时文件和真实进程，不需要模型密钥；行数只作参考统计。

修改教学补丁的贡献者可以先提交候选，再用 `python3 scripts/check-lessons.py --v1-ref HEAD` 验证该提交。日常跟做使用上面的默认命令即可。
