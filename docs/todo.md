# v1 后续整理

本分支只维护 v1 的学习实现；完整版本路线见 main 的需求 Todo。

- [x] 确认 v1 使用 stdin 入口；README 负责配置说明。
- [x] 统一 src 文件的空行：分隔顶层定义与函数内逻辑阶段，同步教学补丁。
- [x] v1 收敛到约 300 行：删除 cli.rs、`.env`、provider 切换、启动信息与参数检查；每个 stdin 行是一个任务；补丁改为六个只做加法的增量。[开发记录](devlog/37-v1-slim.md)
- [x] 用一次真实 DeepSeek 修复任务和一次资料整理任务，为收敛后的 v1 重新取证（`scripts/run-live.py`）；另做了多行会话和 key 隔离检查，见 `evidence/v1/slim-acceptance.md`。
- [ ] 候选（v3，随 TOML 配置一起）：工具安全边界，包括 bash 的 clean env / env allowlist、command approval 和 workspace isolation。v1 有意不做，只在 README 和第 03 章提示风险。
- [ ] v3：`.env`/TOML 配置、provider 预置与凭证；届时再考虑启动信息与更详细的工具状态显示。
- [ ] 后续独立 CLI Help 迭代：层次、排版、高亮多色、纯文本退化和参数声明一致性。[设计边界](cli-help-plan.md)

- [x] 2026-10-25：合并后 v1 的 GA 复核通过，补充真实修复、报告、连续任务和 PTY 证据；见 [GA 复核](../evidence/v1/ga-current.md)。
- [x] 删除过早创建的 milestone/v2，保留历史 v2 标签。
- [ ] 确认 main 重整方案后，以 v1 GA 建立 main 基线，分批加入 context 与 compaction，满意后再建立新的 milestone/v2。
