# 17：规划长任务 Todo 与用户问答工具

2026-10-12。基线为 main 的 d1ef5a4。用户希望把 todo_write、ask_user_question 作为后期常用工具能力纳入合适阶段，并把 llm-space 作为值得学习的参考项目。

todo_write 安排在 v4 的 Skills 与联网长任务阶段，先维护当前任务的完整结构化列表，并在上下文整理后保留最新版。v5 恢复 Todo，再增加 ask_user_question 的待答状态、真实回答、原 call_id 回传、非交互 NeedsInput、取消与恢复。v6 完善 Todo 和问答的界面；v1–v3 的最小四工具范围保持。

只读核对 llm-space 本地 e65eea24c87ee5b91618b7eb50e2af77a30ef042 快照，所引用文件无工作区改动。记录完整列表契约、工具调用参数与专用视图的关系、人工问题排除自动执行，以及研究示例的计划更新。固定 GitHub 源码链接写入 roadmap。此项证据是静态检查，未运行该应用作对照。

README 与 AGENTS 同步阶段摘要。当前生产源码没有变化，新工具、持久化与 UI 仍待相应阶段实现。

验证：cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings、cargo test --locked 通过，61 项 Rust 测试无失败。四份改动文档的相对链接、代码围栏和 git diff --check 通过；四个 llm-space 固定源码入口由 git cat-file 核对。AGENTS 为 8,025 字节，低于 8,192 上限；源码仍为 813 代码行 / 825 物理行。本次是规划更新与既有实现回归，新工具的真实执行验收随对应阶段完成。
