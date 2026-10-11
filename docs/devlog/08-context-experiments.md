# 08：事实对照与真实压缩

日期：2026-10-11。书签：step-08-experiments。

固定 history/faithful/misleading fixtures 比较三种投影：6989/1577/1851 bytes。近期两组保持协议完整却丢失植入事实；误导摘要通过 shape check，却被独立 marker 检查拒绝。新增 loop 集成用例验证摘要调用不计入主轮；候选过大保留旧状态。46 项 Rust 测试通过。

真实尝试完整保留在 evidence/v2：8192 的历史续写错误与无前缀停止；12288 的 2048/4096-token 思考额度耗尽；16384 的修复成功但没触发压缩；以及九次成功整理后因反复核验与尾部过大停止。后者外部源码测试已通过，也仍保留整体未完成结论。

修正来自具体证据：摘要请求独立包装历史；摘要生成额度至少 4096；reasoning.effort=none；提示保留已观察到的完成状态，不因总结而凭空制造待办；任务明确要求简短笔记和完成后结束。JSON 形状断言先红后绿；提示效果通过后续真实任务核对。官方依据：[DeepSeek Responses](https://api-docs.deepseek.com/api/create-response/)、[GPT-6 Luna](https://developers.openai.com/api/docs/models/gpt-6-luna)。模型选择未来需考虑参数能力。

最终 10240-byte 实验成功：11 主轮，5 次摘要提交，agent exit=0，独立 verifier=0，原测试未改，固定任务/AGENTS 保留。运行 25.63 秒，没有做性能基准。[直接证据](../../evidence/v2/20261011T143500013361Z-coding-deepseek.json)。

人工复核确认基线失败、修复后未运行、实测通过的进度得到保留；首份摘要的 AGENTS 已读/未读矛盾及“约五行”来源误归明确记录为局限，不称为无损压缩。[详细教程](../../tutorial/v2/08-context-experiments.md)。

相同最终源码与任务的默认预算对照为 [49152-byte 运行](../../evidence/v2/20261011T143557216175Z-coding-deepseek.json)：9.92 秒、零压缩、修复通过；请求/usage/时延都可查看，但随机轨迹和少量样本不支持压缩更便宜或更快的结论。
