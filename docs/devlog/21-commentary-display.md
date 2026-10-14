# 21：在 main 按轮显示过程说明

日期：2026-10-14。用户确认 v1 保持定稿，将 commentary 展示作为 main 的小体验增量。沿用滚动终端与已有 Progress 回调，不引入新终端框架或 streaming。

model.rs 将每个 assistant commentary message 的文本片段合并成段落。Session::turn 在完整响应通过解析后，为本轮段落发送 Progress::Commentary，然后执行工具。只展示新响应，不重放历史界面；完整 output、JSONL 和 call_id 关联保持。

交互界面以“过程”标签和留白显示原文，模型等待状态改成“step N · 处理中”，最终答案单独输出。单次 -p 模式将过程写入 stderr，stdout 继续只包含最终答案。摘要请求不会触发主任务的过程展示，reasoning 密文不会作为文本显示。

两项 CLI 回归先观察到 commentary 缺失而失败，修复后通过。覆盖 commentary-only、多片段中文换行、工具往返、commentary 与 final 同响应、每轮恰好展示一次、stdout/stderr 分工和原始历史回传。完整 fmt、clippy、默认并行 65 项 Rust、6 项 Python 通过。

使用本地固定 HTTP 响应完成真实 PTY 检查：过程说明、write_file 实际执行、下一段说明和最终答案按顺序出现，每段一次，文件内容独立核对，正常退出。选择并脱敏后的[终端输出](../../evidence/interactive/commentary-pty.txt)与[检查记录](../../evidence/interactive/commentary-rendering.json)已保存；这验证的是终端执行与渲染，不是新增真实模型验收。

独立代码审查通过。生产 Rust 为 836 代码行 / 848 物理行，新增 20 代码行；milestone/v1 保持 ef88b81，运行代码仍为 397/402。
