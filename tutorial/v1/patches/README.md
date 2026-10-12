# 从空目录走到 v1：每一步完整的代码增量

章节正文负责解释为什么这样写，这里保存每一步实际增加、删除和替换的内容。先读 diff，再自己修改；如果只想恢复某一步的参考状态，可以直接应用对应补丁。

这些是教学材料。应用后的练习工程位于你自己的目录，CrabLoop 的正式实现仍只有仓库根目录这一份。

| 补丁 | 起点 → 终点 | 主要变化 | 检查 |
|---|---|---|---|
| [00-probe.patch](00-probe.patch) | 空目录 → 一次 HTTP 往返 | Cargo 工程、固定锁文件、main.rs 探针 | cargo check --locked |
| [01-responses.patch](01-responses.patch) | 探针 → step-01-responses | Config、ModelClient、协议类型/解析、CLI 和协议测试 | cargo test --locked --test protocol |
| [02-file-tools.patch](02-file-tools.patch) | step-01 → step-02-file-tools | schema、三个文件工具、ToolResult 与错误反馈 | cargo test --locked --test tools |
| [03-bash.patch](03-bash.patch) | step-02 → step-03-bash | Bash、双管道采集、超时及对应实验 | cargo test --locked --test tools |
| [04-agent-loop.patch](04-agent-loop.patch) | step-03 → step-04-loop | 历史所有者、串行行动循环与停止 | cargo test --locked --test loop |
| [05-v1.patch](05-v1.patch) | step-04 → v1 | 修复 fixture、独立 verifier、简化接口 | cargo test --locked |
| [06-readable.patch](06-readable.patch) | 原始 v1 → 教学排版副本 | 展开 schema 和协议 JSON；行为不变 | cargo test --locked；cargo fmt --check |

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

05 完成后的 src/ 与固定 v1 标签逐字一致。最后可应用 06，把相同实现展开为更好阅读的 352 代码行（357 物理行）。这个教学副本有明确的来源：v1 + 06-readable.patch；它没有移动 v1 标签，也不冒充 v2 或 v3。

## 验证整条路线

在参考仓库根目录执行：

```bash
cargo fetch --locked
python3 scripts/check-lessons.py
```

第一次 fetch 准备固定依赖；检查脚本随后在临时目录离线应用全部七个增量。00 只构建，01–06 跑各阶段的完整 Rust 测试，05 额外核对 src 是否与 v1 一致，06 检查格式。测试使用本地 HTTP stub 和真实临时文件/进程，不需要模型 Key。

正文里的旧接口属于各自步骤。最终去掉 ModelClient/ToolResult 的变化见 05 补丁及第 05 章的迁移表；06 只负责展开排版。原始 step-05-v1 / v1 标签继续保留当时的完整记录；milestone/v1 分支已加入后续交互增量。
