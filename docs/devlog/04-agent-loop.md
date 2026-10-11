# 04：行动循环与停止原因

日期：2026-10-11。代码书签：step-04-loop。

6 项循环测试在 run 尚未实现时全部失败。随后接入完整 output 回传、串行工具执行、结果关联、最终回答判断、主调用轮数和请求字节上限。

测试在真实临时文件上运行，外部模型由本地 HTTP fixture 代替。它们验证双调用结果、工具失败后的纠正、不完整响应不修改磁盘、推理与 commentary 回传、轮数耗尽和发请求前的体积限制。

CLI 的 Final 退出码为 0，上限停止为 2，配置/HTTP/协议错误为 1。Final 不表示练习项目通过验收，这一点会由 Task 5 的独立 verifier 继续验证。

本阶段完整测试为 9 个协议测试、13 个工具测试、6 个循环测试。cargo test --locked 和 cargo clippy --locked --all-targets -- -D warnings 均通过。
