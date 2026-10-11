# 01：接上原生 Responses

日期：2026-10-11。代码书签：step-01-responses。

从只有文档的仓库创建根目录 Cargo package，固定工具链和五个基础依赖。最初的 ModelClient 只返回“尚未实现”；6 个协议测试全部失败。空 CLI 的两个配置/请求测试也失败。随后实现真实 HTTP 客户端和响应解析，8 个测试全部通过。

检查涵盖 stateless 请求、多个调用的关联、reasoning/phase 保留、commentary 与最终文本、不完整响应、缺失或重复 call_id、空响应、HTTP 500，以及 CLI 配置。fake server 只代替外部模型，HTTP 和解析路径使用真实代码。

用户提供的配置使用 OPENAI_BASE_URL，因此增加了 URL 前缀的支持；CRABLOOP_API_URL 继续表示优先的完整端点。执行测试时只给子进程注入所需凭证，不写回 .env，不把其他服务凭证带入记录。

模型优先级按最新指令确定：DeepSeek V4.1 Flash 主用，API id 为 deepseek-flash；TeamoRouter 的 gpt-6-luna 为测试备选。DeepSeek [官方模型页](https://api-docs.deepseek.com/quick_start/pricing/)确认该别名及 Responses 支持。首次指令更新前的 gpt-6.1-sol 文本探测收到指定标记；它只作为早期连通性记录，不替代主用/备选模型验收。

当前阶段只验证一轮请求与协议解析。真实工具闭环和压缩将在相应任务中分别验收。

新增 OPENAI_BASE_URL 子进程回归后，协议测试为 9 项。主用 deepseek-flash 和备选 gpt-6-luna 均通过真实文本标记探测；记录见 [协议探测](../../evidence/v1/protocol-smoke.json)。这仍不表示四工具闭环已通过。
