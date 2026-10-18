# v1 后续整理

本分支只维护 v1 的学习实现；完整版本路线见 main 的需求 Todo。

- [x] 确认 v1 使用 stdin 入口；README 负责配置说明。
- [x] 统一 src 文件的空行：分隔顶层定义与函数内逻辑阶段，同步教学补丁。
- [x] v1 收敛到约 300 行：删除 cli.rs、`.env`、provider 切换、启动信息与参数检查；每个 stdin 行是一个任务；补丁改为六个只做加法的增量。[开发记录](devlog/37-v1-slim.md)
- [x] 用一次真实 DeepSeek 修复任务和一次资料整理任务，为收敛后的 v1 重新取证（`scripts/run-live.py`）；另做了多行会话和 key 隔离检查，见 `evidence/v1/slim-acceptance.md`。
- [ ] v3：`.env`/TOML 配置、provider 预置与凭证；届时再考虑启动信息与更详细的工具状态显示。
- [ ] 后续独立 CLI Help 迭代：层次、排版、高亮多色、纯文本退化和参数声明一致性。[设计边界](cli-help-plan.md)
