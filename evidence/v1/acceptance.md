# v1 验收记录

日期：2026-10-11。核心代码 314 行，物理行 319；src 下所有 Rust 均参与统计。四工具、Responses、串行 loop 与停止边界已完成。

| 证据层次 | 结果 |
|---|---|
| 确定性协议/loop | 9 项协议、7 项循环测试通过；包括一次 scripted 四工具修复 |
| 真实本地工具 | 13 项文件/Bash 测试通过，含唯一匹配、UTF-8、双管道限额和直接子进程超时 |
| 独立报告 verifier | 3 项 Python 回归通过；允许正确附加字段，拒绝错误事实 |
| 格式与 lint | cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings 通过 |
| DeepSeek 真实修复 | deepseek-flash（V4.1 Flash），10.70 秒；原测试先失败后通过，独立 verifier 4/4，tests.rs 未修改 |
| 测试备选真实修复 | gpt-6-luna，经 TeamoRouter，149.43 秒；同样通过，未修改原测试 |
| DeepSeek 通用资料任务 | 9.09 秒；生成结构化报告与来源说明；修正 verifier 后原产物通过 |

直接证据：[DeepSeek 修复](20261011T134831569033Z-deepseek.json)、[Luna 修复](20261011T134843235452Z-luna.json)、[报告原始结果](20261011T135900476081Z-report-deepseek.json)、[报告复核](20261011T135900476081Z-report-deepseek-recheck.json)。早期的 [11.25 秒修复](20261011T133150486826Z-deepseek.json) 来自规模整理前，仍保留。

报告原始 passed=false 来自过严的对象相等检查，模型多给了正确的 total。复核修正验收规则，不重新生成。单次时延只是观察，没有进行性能基准或成本比较。

记录 source_commit 是运行时最近的提交；当时工作区尚有变更，source_sha256 才标识实际运行的各源文件。原始 stdout/stderr 在忽略的 .crabloop/live 下，公开结果仅保留选择过的任务证据，不含凭证。

边界：路径检查不等于 sandbox；Bash 只治理直接子进程；文件写入不提供中途 I/O 失败回滚；请求限制是序列化字节上限，不是精确 token 计数；无恢复、自动重试、Skills 或内建网络工具。v1 tag 固定这些行为，v2 继续研究历史和压缩。
