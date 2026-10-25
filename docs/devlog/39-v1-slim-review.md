# PR #1 精简方案复核

日期：2026-10-25。复核起点为 `codex/v1-slim` 的 `3e4c90d`，目标分支为 `milestone/v1`。用户授权分析删减是否合适，并在原 PR 上修正。

## 判断

保留本次精简方向：三项环境配置、统一逐行 stdin、简短工具展示、`Stop::Final(String)` 和 `Reply.final_text: Option<String>` 都能减少外围状态。六个按章节增加代码的补丁也避免了末章一次替换阶段接口。`.env`、provider presets、banner、完整 CLI 与终端编辑后移，作为 v1 的范围取舍合理。

逐行管道是明确的接口变化：原本整段管道输入是一个任务，现在每个非空行各是一个任务。README 和教程保持一致，不再引入第二套读取路径。历史仍由 main 持有，工具错误仍以原 call_id 回传。

## 复现与修正

- 原版接受同一响应内的空、重复 call_id。用两个相同 ID 的 write_file 复现：即使最后请求出错，文件已经从 ORIGINAL 被改成 SECOND。恢复响应解析阶段的 ID 检查，并增加协议与文件不变回归。
- 原版把非 assistant message 的文本当成最终答案，也不区分 content type；缺少 output_text.text 时静默接受。恢复 role 与 output_text 的分类，使用 tagged Content enum；原始 output 继续完整保存。
- 文件读取改为 take(FILE_BYTES + 1)，先判断实际字节数，再解码；metadata 不能代替实际有界读取。增加三行保持读取上限，不恢复 Limits 类型。
- 最新用户约定将行数作为教学参考；修正 PR 中重新引入的硬上限与无关的 v2/v3 规模变化。README 的 sandbox 说明压成一段，保留实际工具权限和环境继承事实。

核心由 PR 初稿的 304 代码行 / 350 物理行调整为 324 / 370，仍为四个 Rust 文件；没有把 loop 或工具实现移到 src 外。examples/probe.rs 是复用同一模型函数的教学入口。

## 验证与证据

- PR 原始基线：fmt、clippy、29 项 Rust 与 6 项 Python 测试通过。
- 新增的四项回归先在原始实现失败，修正后通过；最终 fmt、clippy、33 项 Rust 与 6 项 Python 测试通过。
- 重新生成六个教学补丁，逐一检查没有删除行；每阶段 fmt/tests 通过，最终所有生成文件与工作区一致。代码摘录、Markdown 链接/围栏和 Bash 示例核对通过。
- PR 已有的 2026-10-18 DeepSeek 修复、报告与多行会话证据保留。可用记录中的 src SHA256 与 `79d0d85` 核对一致；报告 verifier 的负责人应忠实于中文 fixture，PR 对此的修正合理。
- 本次没有发起新的真实模型运行，离线回归不替代这次修正后的 live 验收。远端未提供该 PR head 的 CI status/workflow run，结论依据本地检查与已提交的历史证据。

更新原 PR 分支及说明，不合并 PR，不修改 main、milestone/v1 或历史标签。
