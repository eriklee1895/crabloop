# 19：在 main 加入调试实践指南

日期：2026-10-14。用户确认调试指南只加入 main，保留 v1 的最小学习范围；内容覆盖代码单步、网络请求观察与工具选择。

docs/debugging.md 以 main 的 Session::turn 为执行入口，先复用 tool_error_then_corrected_call_receives_actionable_feedback 的固定响应，再观察实际模型任务。把模型响应、call_id、工具执行、下一次输入和独立任务验证连接起来，补充 async 单步、变量查看、超时和已有 JSONL 的边界。

macOS 的交互抓包优先介绍 Proxyman，脚本化实验再介绍 mitmproxy。核对官方文档及锁定依赖源码后发现两个重要条件：CrabLoop 的 dotenvy 只把 .env 当配置数据读取，不会为 reqwest 设置 HTTPS_PROXY；reqwest 0.13.5 关闭 default features，没有开启 system-proxy，但仍读取进程代理环境变量。rustls 默认通过平台校验器验证证书，因此指南采用 macOS CA 信任与目标域名 SSL Proxying，没有照搬 Proxyman Rust 示例中的关闭证书验证。

## 检查与进度

- [x] 核对 main 源码、JSONL 事件和官方工具操作文档。
- [x] 写入指南、README/教程导航和 main 的 agent 指南。
- [x] 固定测试通过，实际出现 bad 调用的 ok:false、good 调用的 ok:true 和第三轮结束；本地假代理收到 CONNECT llm-debug.invalid:443 HTTP/1.1，返回 502 后进程退出 1，没有连接外部模型。
- [x] 文档本地链接、代码块和 AGENTS.md 体积检查通过；cargo build、fmt、clippy、61 项 Rust（单线程）和 6 项 Python 通过。

发布以本次 main 的 Git 提交为准；milestone/v1 不更新。

本次不修改生产 Rust、Cargo 配置或测试文件，不安装抓包工具、不改系统代理和证书，不发真实模型请求。IDE 单步与 Proxyman HTTPS 解密操作依据源码和官方文档，尚未在 GUI 中实测。
