# v1 GA 后的 main 演进建议与导航清理

日期：2026-10-25。用户要求删除提前建立的 milestone/v2，并讨论 main 以 v1 为基准分批增加 v2。v1 的本轮 GA 证据已推送到 milestone/v1：运行基线 `9a8077e`，证据与分支策略提交 `e1baa80`。

已删除本地和远端 milestone/v2；旧 v2 标签保持在 `5a5e96e`。main 的 README、教程入口和 Git 工作流改为引导稳定 v1，旧 v2 资料明确为历史参考；不再提供已删除分支的活动链接。

main 重整建议见 [Git 工作流](../git-workflow.md)。建议将运行代码与依赖对齐 v1 GA，保留 main 的品牌资源、CI 与 roadmap；使用父节点为 `[v1 GA, 旧 main]` 的基线提交：保留旧历史和 first-parent 学习路线，同时允许 main fast-forward 更新。之后分批添加 context state、投影、compaction 与任务/教程验收，满意后再建立新的 milestone/v2。

该方案尚待确认和实施。本次 main 只更新文档与导航，src、Cargo、测试和脚本保持 `439b081` 的内容，没有恢复旧 v2，也没有替换 main 后端。

文档校验、AGENTS.md 的 8 KiB 限制检查及 main 的 fmt、clippy、65 项 Rust 与 6 项 Python 测试通过。未运行 main 的真实模型验收；GA 结论属于 milestone/v1。
