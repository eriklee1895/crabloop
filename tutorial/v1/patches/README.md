# 从空目录走到 v1：每一步完整的代码增量

v1 的学习代码统一为 `milestone/v1`。直接阅读该分支，或从空目录顺序应用这里的七个增量，最后得到同一份实现。章节正文负责解释为什么这样写，补丁保存每一步实际增加、删除和替换的内容。

这些是教学材料。应用后的练习工程位于你自己的目录，CrabLoop 的正式实现仍只有仓库根目录这一份。

| 补丁 | 起点 → 终点 | 主要变化 | 检查 |
|---|---|---|---|
| [00-probe.patch](00-probe.patch) | 空目录 → 一次 HTTP 往返 | Cargo 工程、固定锁文件、main.rs 探针 | cargo check --locked |
| [01-responses.patch](01-responses.patch) | 探针 → step-01-responses | Config、ModelClient、协议类型/解析、CLI 和协议测试 | cargo test --locked --test protocol |
| [02-file-tools.patch](02-file-tools.patch) | step-01 → step-02-file-tools | schema、三个文件工具、ToolResult 与错误反馈 | cargo test --locked --test tools |
| [03-bash.patch](03-bash.patch) | step-02 → step-03-bash | Bash、双管道采集、超时及对应实验 | cargo test --locked --test tools |
| [04-agent-loop.patch](04-agent-loop.patch) | step-03 → step-04-loop | 历史所有者、串行行动循环与停止 | cargo test --locked --test loop |
| [05-v1.patch](05-v1.patch) | step-04 → 综合任务的中间步骤 | 修复 fixture、独立 verifier、简化接口 | cargo test --locked |
| [06-v1-entry.patch](06-v1-entry.patch) | 综合任务 → milestone/v1 | 展开协议结构、cwd 工作区、stdin 单任务输入和 .env 配置 | cargo test --locked；cargo fmt --check |

00 的 diff 包含 Cargo.lock，因此较长；先关注 Cargo.toml 和 src/main.rs。01 的完整增量包含正文里只解释了关键部分的 src/lib.rs、src/model.rs、src/main.rs 和 tests/。这样可以看到 Config 从哪里来，以及入口怎样接上协议层。

## 准备两个明确的位置

先在 CrabLoop 参考仓库根目录记下它的位置，然后创建一个空练习目录。这套操作不会切换参考仓库分支：

```bash
export CRABLOOP_SOURCE="$PWD"
mkdir -p playground
mkdir playground/v1-lesson
cd playground/v1-lesson
git init -q
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/00-probe.patch"
cargo check --locked
```

v1-lesson 必须是尚未使用的目录；已经有同名练习时换一个名字。00 构建不需要模型凭证；真正运行探针时按第 01 章配置环境。

读完第 01 章有关 Config、响应解析和错误的解释，再在练习目录运行：

```bash
git apply --check "$CRABLOOP_SOURCE/tutorial/v1/patches/01-responses.patch"
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/01-responses.patch"
cargo test --locked --test protocol
```

后面的章节用相同方式应用 02、03、04、05，按表中命令检查。补丁是顺序增量，不能直接把 04 应用到空目录。如果已经手工修改了代码，先读 diff 并合并自己的改动；不要覆盖练习成果来强行通过 patch 检查。

05 是综合任务的中间步骤。继续应用 06，完成当前 v1 的入口和协议排版，得到与 `milestone/v1` 逐字一致的全部 src 文件：399 代码行、460 物理行。其中 CLI 为 63 代码行，其他实现合计 336 行。阅读和练习都以这个终点为准。

## 验证整条路线

在参考仓库根目录执行：

```bash
cargo fetch --locked
python3 scripts/check-lessons.py
```

第一次 fetch 准备固定依赖；检查脚本随后在临时目录离线应用全部七个增量。00 只构建，01–06 跑各阶段的完整 Rust 测试，05 核对历史步骤来源，06 核对全部 src 与 `milestone/v1` 一致，并检查格式。行数统计仅供参考。测试使用本地 HTTP stub 和真实临时文件/进程，不需要模型 Key。

正文里的旧接口属于各自步骤。去掉 ModelClient/ToolResult 的变化见 05 补丁及第 05 章的迁移表；06 接上当前 v1 的简洁启动。历史标签和开发记录保留当时的证据，学习时无需选择另一份 v1。

修改 v1 的贡献者可以在提交候选分支后执行 `python3 scripts/check-lessons.py --v1-ref HEAD`，先验证重建终点与该提交一致，再推进 milestone/v1；默认命令仍校验正式学习分支。

2026-10-14 默认预算增量在最后一个补丁中将请求预算提高到 4 MiB、max_output_tokens 提高到 131,072；前面的历史步骤与 v1 标签继续保留当时的数值。
