# 06：运行记录与当前输入分开

日期：2026-10-11。书签：step-06-context。

7 个新测试先因 context/journal 尚未实现而失败。实现固定任务、根 AGENTS、完整交互组、请求 byte 计量、JSONL create_new/flush 后，7 项全部通过，既有 29 项回归通过，clippy 通过。

loop 先在局部 round 内执行所有工具，然后验证并追加完整组。模型原始 output、reasoning、phase 保留；日志额外保存成功解析的原始响应 envelope。HTTP/协议失败仅记录错误，失败 HTTP body 不存档。这是对最初“完整原始响应”措辞的收窄，避免扩大模型模块的接口；工具意图与实际结果仍分别记录。

最小投影实验保留 task/instructions，仅删除完整旧组；它不产生协议孤儿项，却会丢失旧组事实。第 08 章继续构造可量化的事实损失案例。

本版本日志位于工作区 .crabloop，用于观察单次 run；最新路线明确将 home 下可恢复 sessions 留到 v5。两者不混称。
