# v1 GA 审查与验收

日期：2026-10-14。审查基线为 925eadc，最终实现以 `v1-ga` 标记，唯一学习入口继续是 milestone/v1。验收标准是清晰、够用、可教学；不追求覆盖所有边缘情况。

## 结论与范围

最终审查通过，可以作为教学 v1 GA 定稿。运行代码的独立复审、格式、回归、真实任务及教学重建检查均已完成。保留原生 Responses、手写模型—工具—反馈循环、read_file/write_file/edit_file/bash 四工具、单任务入口及明确预算与错误。没有引入新框架、抽象层、额外工具或另一份 v1 实现。

五个平铺文件的职责合理：main.rs 接线与退出；cli.rs 读取任务和配置；lib.rs 持有历史、公共类型、循环与停止；model.rs 处理 HTTP 和 Responses；tools.rs 定义并执行四工具及有限输出采集。当前规模下无需再拆目录或文件。

## 格式与规模

实际执行 cargo fmt，再以 cargo fmt --check 核对；使用 max_width=100、use_small_heuristics=Max，协议 JSON 保持展开。所有生产 Rust 都在 src 下，按 scripts/count-lines.py --max 399 统计：

| 文件 | 代码行 | 物理行 |
|---|---:|---:|
| src/cli.rs | 71 | 72 |
| src/lib.rs | 84 | 85 |
| src/main.rs | 14 | 14 |
| src/model.rs | 72 | 73 |
| src/tools.rs | 156 | 158 |
| 合计 | **397** | **402** |

CLI 之外的实现为 326 代码行。相对审查基线，总代码减少 1 行；没有通过压缩格式或移动生产逻辑满足预算。

## 收尾修复

1. 同一响应混合 commentary 与 final_answer 时，展示最终文本，完整原始 output 继续回放。回归覆盖两种排列顺序。
2. 空任务与 stdin EOF 在读取模型配置之前结束，不要求 Key。任务输入集中到一次 match，减少重复分支。
3. 已存在的非普通文件在公共文件路径检查处拒绝，避免 write_file 在已有 FIFO 上等待。失败作为工具结果以原 call_id 回传。

三项新增回归都先在旧代码上失败，再通过修复。默认 DeepSeek 配置测试也改为发出本地 stub 请求并核对实际模型与凭证选择。

## 验证结果

| 检查 | 结果 |
|---|---|
| cargo fmt --check | 通过 |
| cargo clippy --locked --all-targets -- -D warnings | 通过 |
| cargo test --locked | 默认并行运行，43 项 Rust 通过 |
| Python 验收工具回归 | 4 项通过 |
| 行数门禁 | 397 <400，通过 |
| 本地 PTY | 中文单任务、无凭证 EOF 通过，正常任务只发一次本地 HTTP 请求 |
| 教学增量重放 | 七步通过；末步 43 项 Rust、fmt、行数门禁通过，全部 src 与 GA 源码一致 |
| DeepSeek 真实修复 | 17.74 秒，原测试先失败后通过、tests.rs 未改、独立 verifier 通过 |
| DeepSeek 真实资料任务 | 13.25 秒，结构化报告独立 verifier 通过 |

教学重放使用 `RUST_TEST_THREADS=1 python3 scripts/check-lessons.py`，各测试阶段为 9、17、22、28、29、43 项；历史步骤保持原样，最后一步接到 GA 实现。

本次选择后的证据：[修复任务](20261014T031057603442Z-coding-deepseek.json)、[资料任务](20261014T031158677214Z-report-deepseek.json)、[PTY](ga-terminal.json)。三个记录的五份生产源码哈希均与最终实现核对。live 记录中的 source_commit 是运行时已有的基线提交；当时修复尚未提交，实际运行代码由 source_sha256 标识。时延是单次观察，不是性能基准。

真实模型负责选择动作；独立 verifier 负责检查任务结果。Luna 仍保留历史成功记录，本轮未重新验收。

## 接受的边界

v1 不提供 OS sandbox、符号链接隔离、原子写入回滚、完整后代进程清理、恢复、重试或 streaming。已有文件检查不解决并发替换竞态。请求预算使用序列化字节数。上述边界保持公开，不纳入这次定稿扩展。

历史 100ms Bash 测试出现过调度敏感失败；本轮默认并行检查通过，未修改生产超时机制或削弱断言。开发用 live driver 的 .env 优先级和解析范围与 runtime 不完全相同，是非阻塞维护项；本次实际选择的 provider/model 与源码哈希已经记录。
