# 07：候选摘要与状态提交

日期：2026-10-11。书签：step-07-compaction。

新增 6 项压缩测试，在缺少 prepare 时失败，再实现阈值、完整前缀/两组尾部、独立摘要请求、大小/栏目校验、候选提交与最多 15 次摘要尝试。42 项 Rust 测试、clippy 通过。

接口小幅调整为 Result<Option<Vec<Value>>>：Some 是准备好的 input，None 表示预算无法容纳，Err 表示请求或摘要校验失败。主 loop 对 None 保持 ContextLimit。测试中的 1-byte 非法配置改成合法 4096-byte 预算加过长任务，继续验证发请求前停止。

第一次 8192-byte 真实实验触发了摘要，返回的却是继续读文件的动作文本，缺五类摘要信息。保留[失败记录](../../evidence/v2/20261011T142240455541Z-coding-deepseek.json)。新增回归确认摘要请求只有一个 JSON 数据消息；原任务和历史作为数据字段，替代原生对话继续。失败检查先红，修正请求构造后变绿。

随后一次 8192-byte 实验在两组历史后达到 soft，没有可移出的前缀，报告 ContextLimit。保留[停止记录](../../evidence/v2/20261011T142404666716Z-coding-deepseek.json)。这是预算与组大小的边界，不删除半组凑成功。第 08 章继续校准并做成功/损失对照。

JSONL 以 request/response 与 summary_request/summary_response 区分调用和 usage；compaction 记录 before/after bytes、移出组数及摘要。实际代码仍约 500 行。
