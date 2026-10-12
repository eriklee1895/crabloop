# 16：把可观测性和终端交互拆成可学习的增量

2026-10-12。基线为 main 的 651cb2b。本次更新规划与入口说明，运行实现继续保持 813 代码行 / 825 物理行。

用户希望通过可见的 step 理解 Harness 执行过程，并提出通过 config 使能 Langfuse telemetry。路线图新增 v3 配置完成后的独立观测专题，主版本编号保持；生命周期 observer 关联用户任务、主循环轮次、工具 call_id、摘要与停止，Rust 接入候选为 OpenTelemetry/OTLP。

本次核查 Langfuse v4 官方文档与 Rust OTel 已发布版本，记录 0.33.1 候选和 v4 传输、属性传播、完整 span 的要求。只确认本地 Langfuse 变量声明，未验证账号、部署版本或真实入库。遥测默认关闭，后台导出有界，本地 JSONL 保留执行记录。

v6 拆为输出渲染、slash command 交互、完整布局三个可运行增量。thinking 状态与可读推理分别处理，实时 Bash 输出需要执行层事件。当前 step 表示进入主循环轮次，末尾统计成功主模型响应；工具和摘要调用另行计数。教学验收要求界面中的执行过程能对应到源码位置。

Rig 明确为可跳过的选修对照。v1–v7 保持手写核心，是否采用库取决于具体重复工作、协议保留和维护收益。README 与 AGENTS 同步规划摘要；AGENTS 通过精简现有表述维持字节上限。

验证：cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings、cargo test --locked 通过，61 项 Rust 测试无失败。四份改动文档的相对链接、代码围栏、凭证字面量检查和 git diff --check 通过；AGENTS 为 7,960 字节，低于 8,192 上限。scripts/count-lines.py 确认源码仍为 813/825。本次验证文档一致性与既有实现回归，真实 Langfuse 轨迹验收留给观测专题实施。
