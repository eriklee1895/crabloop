# v1 scope

v1 用约 300 行 Rust 教会读者一个完整的 agent loop：请求模型 → 执行工具 → 回传结果 → 继续或停止。行数是规模参考。

## 核心能力

- 使用非流式 Responses API，保留完整 output 与 call_id 关联。
- 提供 read_file、write_file、edit_file 和 bash；工具错误回传模型。
- 设置请求、输出和轮数限制，区分 Final、MaxTurns、ContextLimit 与运行错误。
- 终端支持连续提问，main 持有历史，run_turn 借用历史执行当前任务；管道输入只运行一次。
- 从启动目录读取 .env，工作目录为 cwd，不接受命令行参数。

## 后续范围

v2 增加 context 管理与 compaction；v3 增加基础 SSE、配置和 model catalog。输入编辑器、session 持久化、恢复与 memory 留给后续版本。

## 学习与验证

学习代码统一为 milestone/v1；七个教学补丁重建同一份 src。历史 tags 保存章节步骤和验收快照。

读者应能沿代码解释历史由谁维护、工具由谁执行、失败怎样进入下一次请求，以及程序为何停止。运行修复或资料整理任务后，独立检查文件和测试结果。

开发约定见 [AGENTS.md](../AGENTS.md)，连续对话的实现与验证见[开发记录](devlog/33-v1-simple-chat.md)，CLI Help 规划见[专门文档](cli-help-plan.md)。
