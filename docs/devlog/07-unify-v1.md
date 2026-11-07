# 统一 v1 学习分支

日期：2026-10-12。用户要求不再单独保留 v1.1，使用优化直接归入 milestone/v1。本次把已有交互实现快进到 v1 分支，更新 README、教程和 AGENTS 中的入口名称；原始 v1 / step-05-v1 标签仍保存最小循环，作为历史实验基线。

运行代码没有变化：完整 v1 分支仍为 571 代码行 / 581 物理行，支持默认启动、.env、-p 和多轮 Session，无 v2 压缩与 journal。Cargo 的 0.1.1 包版本只是构建元数据，与学习分支名分开。

复核时，原支线的 cli_validates_configuration_before_network_access 因仓库根目录存在 .env 而失败。三个旧 CLI 协议测试未设置启动目录；补上与 main 一致的临时 current_dir，避免读取开发者配置。原支线工作树没有 .env 时通过的记录仍作为当时事实保留。

本次检查包括全套 Rust/Python 回归、fmt、clippy、源码与原交互提交一致性，以及删除旧名称前的引用保留检查。具体发布结果见 main 的统一版本记录。
