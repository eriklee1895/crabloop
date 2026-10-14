# main 同步 v1 图文教程

日期：2026-10-14。用户确认提交 v1 教程，并要求检查 main 是否需要同步。v1 改稿已提交推送，随后合入简练语言约定；本次同步来源为 `milestone/v1` 的 `7225d5a`。

将 v1 的 00–06 章、12 张 Mermaid 图、补丁导航和样章/全套整理记录同步到 main。保留 main 的 v2 章节、debugging 入口、完整 Session 与 compaction 实现。根 README 与教程入口区分各阶段；第 06 章的启动链接明确指向 v1 README。旧 05b 交互专题保留为历史选读，修正其中对当前 v1 的过时描述。

写作约定补入已确认的样章、配图和阶段说明要求，保留 main 现有的完整增量与术语约定。AGENTS.md 更新教程状态，保持低于运行时的 8 KiB 限制。

验证：

- 同步章节与 v1 原文逐一核对，12 张 Mermaid 源码完全一致；第 06 章仅调整跨分支 README 链接。链接与代码围栏检查通过。
- 从 main 运行七步教程回放，最终 src 与 `milestone/v1` 一致；最终阶段 44 项 Rust、6 项 Python 和 fmt 通过。
- main 的 fmt、clippy、65 项 Rust 与 6 项 Python 测试通过。
- main 的 src、测试、运行脚本、Cargo 配置及锁文件未改动；没有把 v1 后端合并覆盖 main，也没有发起新的真实模型运行。
