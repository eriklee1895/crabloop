# 11：直接运行，再输入任务

日期：2026-10-12。用户实际执行 cargo run 后遇到 usage 错误。根因是入口仅接受精确的四项 argv，同时模型设置仍只能从进程环境读取，用户已准备的 .env 没有被运行时加载。

本次把启动从学习阶段的显式命令改为日常入口：无参数默认当前目录，终端提示输入一个任务；也接受直接任务参数、管道输入，以及原有 --workspace DIR TASK。--help 不要求凭证，EOF 退出而不发模型请求。连续多轮会话仍未实现。

CLI 配置和输入逻辑移到 src/cli.rs，运行核心继续接收同一个 Config。加入固定版本 dotenvy 0.15.7，通过 from_path_iter 把 .env 读成数据，再按进程环境优先解析；不调用修改全局环境的加载函数，也不执行 shell。解析错误不回显可能含 Key 的原文。

结合用户已有的两套配置，默认优先 DeepSeek；显式模型/端点或进程级 OpenAI 设置沿用通用模式，CRABLOOP_PROVIDER 可明确覆盖。DeepSeek 使用自己的 Key，通用模式使用 OPENAI_API_KEY；模型缺省值遵循本项目主用 deepseek-flash / 备选 gpt-6-luna。启动时显示所选 provider、模型与工作目录。已有 .env 未被修改。

8 项启动回归先全部失败，再通过：无参数 stdin、直接任务、help/EOF、dotenv/环境覆盖、主模型默认值、provider 凭证匹配、错误脱敏、指定目录后输入任务。测试服务器额外核对 Authorization，所有凭证均为假值。测试 fixture 中一条未加引号的 shell 形状字符串不符合 dotenv 语法，修正为合法的字面量后验证没有执行命令。

原有 CLI 测试改在独立临时目录运行，避免加载开发者真实 .env。当前 54 Rust + 6 Python 测试通过，fmt/clippy 通过。main 为 628 代码行、637 物理行；历史 v1/v2 标签与教学补丁保持不变。

在本机终端实际运行 cargo run，正确显示 deepseek/deepseek-flash 和当前目录。输入“只回复 CRABLOOP_STARTUP_OK，不调用任何工具”，一轮真实模型请求正常返回标记并以 Final 结束；未打印或发布凭证。证据见 [启动 smoke](../../evidence/startup/smoke.json)。这只证明本次启动与一条真实回复，不扩大为任务成功率结论。
