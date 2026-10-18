# v1 scope

v1 用约 300 行 Rust（上限 400 行）教会读者一个完整的 agent loop：请求模型 → 执行工具 → 回传结果 → 继续或停止。只保留演示这条回路所必需的代码，其余能力留给后续版本。

## 核心能力

- 使用非流式 Responses API，保留完整 output 与 call_id 关联；外层响应和需要读取的输出项用 serde 类型解析，其余项原样回放。
- 提供 read_file、write_file、edit_file 和 bash；工具错误以 `ok: false` 回传模型。bash 子进程不继承 `OPENAI_API_KEY`。
- 设置请求字节、输出 token 和每个任务的轮数上限，用 `Stop` 区分 Final、MaxTurns、ContextLimit；运行错误由 `Result` 返回。
- 一个入口：stdin 的每个非空行是一个任务，终端与管道同一路径；main 持有历史，run_turn 借用历史执行当前任务。
- 配置只有三个环境变量：OPENAI_API_KEY、OPENAI_BASE_URL（默认 OpenAI）、CRABLOOP_MODEL；工作目录为启动目录，不接受命令行参数。

## 后续范围

v2（约 500–600 行）增加 context 管理与 compaction；v3（1000 行以上）增加基础 SSE、`.env`/TOML 配置、provider 预置、凭证与 model catalog。输入编辑器、启动信息、CLI Help、session 持久化、恢复与 memory 留给后续版本。

## 学习与验证

学习代码统一为 milestone/v1；六个只做加法的教学补丁从空目录重建同一份 src，`scripts/check-lessons.py` 负责核对。

读者应能沿代码解释历史由谁维护、工具由谁执行、失败怎样进入下一次请求，以及程序为何停止。运行修复或资料整理任务后，独立检查文件和测试结果。

开发约定见 [AGENTS.md](../AGENTS.md)，本次收敛的实现与验证见[开发记录](devlog/37-v1-slim.md)，CLI Help 规划见[专门文档](cli-help-plan.md)。
