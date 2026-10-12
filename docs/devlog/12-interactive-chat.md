# 12：启动之后，可以接着说

日期：2026-10-12。用户指出任务参数不应必填，基础多轮交互应从早期版本提供，单次任务使用 -p，并希望 README 展示真实运行。

本次加入轻量滚动终端界面，使用 rustyline 18.0.1 编辑输入和浏览进程内输入历史。默认终端启动进入聊天，-p/--print 单次执行；旧位置参数、完整 stdin 管道保留，--chat 可强制逐行聊天。/new、/help、/exit 在本地处理。空会话退出不会请求模型或创建 journal。

src/lib.rs 将跨提问的状态放到 Session 中，turn 复用原串行循环，run 保留为单次包装。最终回答也进入历史；Progress 回调分别给交互入口和单次 JSON 轨迹提供展示。v2 用 task_at 记录当前问题的位置：旧提问归入历史，新问题仍独立保留；压缩候选同步移动锚点，避免第二次提问被丢掉或提前到旧回复前面。

传输、协议、记录错误或 Ctrl-C 中断后，Session 拒绝再次发送任务，要求先核对文件并 /new。工具错误仍作为原 call_id 的结果反馈。当前取消没有回滚保证，也不实现持久化恢复或 Bash 进程树清理。

会话测试先因缺少 Session/begin_task 编译失败，再验证最终回答保留、完整工具组、失败后重建；CLI 测试验证 -p 的 stdout、多轮与 /new、空会话。另用实际 PTY 和本地 HTTP fixture 验证 Ctrl-C 在请求期间生效。clippy 发现一处可合并 if，修正后通过。Python discover 首次使用了错误文件模式，没有运行测试；改为 *_test.py 后实际运行 6 项。

DeepSeek 真实演示完成两轮相关任务，6 次主请求、5 次工具调用，两个 Final。独立检查 hello.txt 为 Bonjour, CrabLoop!，第二个用户回合含有第一轮问题和回答。录制来自真实 PTY，使用 agg 1.9.0 渲染 GIF；核查关键帧后放入 README。详见 evidence/interactive/，不公开完整 journal 或凭证。

当前生产源码为 813 代码行 / 825 物理行，含 CLI/UI、v2 context 和全部 src，原 v1/v2 的行数只指各自固定快照。基础交互提前，v6 调整为 streaming 与完整布局。v1.1 从原 v1 补出独立交互学习快照，不回写历史标签；教程增加 05b，给出 v1..v1.1 的完整 diff 学习入口。

最终检查：61 Rust + 6 Python 测试通过，cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings、git diff --check 通过。独立审查复核了历史顺序、压缩锚点和取消路径，未发现阻塞项。v1.1 回移快照单独通过 43 Rust + 3 Python 测试，其真实模型验收不与 main 的演示混为一谈。
