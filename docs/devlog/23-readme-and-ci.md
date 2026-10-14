# 23：首页展示与基础 CI

日期：2026-10-14。用户批准小螃蟹、Banner、一行徽章和基础 CI，并要求版本命令先记入需求 Todo。

README 增加 CI、Rust、v1 <400 行、中文教程四枚徽章与简短导航；v1 行数链接到 milestone/v1，CI 徽章固定 main。尚未选择许可证，不添加 License 徽章。

内置 imagegen 生成吉祥物和 Banner 候选。第一版循环箭头被用户否决；后续探索七种 Banner 风格，素材暂留本地忽略的 .crabloop/branding-preview，未定稿图片不进入 Git 或 README。选定后再把最终素材复制到 docs/assets 并接入首页。

.github/workflows/ci.yml 在 main push、PR 与手动触发时运行单个 Ubuntu 24.04 job。使用固定 SHA 的官方 checkout v7，依照 rust-toolchain.toml 加载工具链，执行 fmt、clippy、Rust 与 Python 离线测试。无模型凭证、不发真实 LLM 请求、不增加 release 流程。

本地同组命令通过：65 项 Rust、6 项 Python，fmt/clippy 无错误。独立只读审查未发现需修改问题；GitHub 托管的 [首次 push CI](https://github.com/eriklee1895/crabloop/actions/runs/38520501082) 已通过，源码提交为 0cab0db，Linux job 用时 1 分 6 秒。

REQ-021 记录 version 子命令与 --version/-V 的最小行为，包版本不等同教学阶段。REQ-022 留待出现二进制分发需要时再规划自动 Release。生产 Rust 与 milestone 分支未修改。
