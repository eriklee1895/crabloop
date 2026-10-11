# 06：历史还在，但不必每次全部带上

v1 把一次模型输出和工具结果依次放进 Vec，然后整份重发。任务很小时，这样直观，也很容易检查。读过几份长文件以后，再看 input，你会发现里面既有仍然重要的约束，也有已经用过的终端输出。

这时先别急着“删旧消息”。我们要先弄清楚删掉的最小单位是什么。

## 一次动作要有完整的来回

打开 src/context.rs 的 ContextState。原任务 task 和固定 instructions 独立保存；rounds 里的每个 Vec，装着一轮主模型的所有 output，以及这一轮每个调用对应的结果。reasoning 与中间说明也跟着这一组走。

在 src/lib.rs 中，原来直接追加 input 的位置，现在先收集 round。工具逐个执行完后，调用 `state.append_round(round)?`。它核对 call_id：出现过的调用必须有结果，结果前必须有对应调用，重复结果也会报错。检查成功才把整个组交给历史。

运行 `cargo test --locked --test context rejects_orphan_missing_duplicate_or_out_of_order_results`，可以看到故意留下半组时，程序拒绝把它加入上下文。这样以后移走旧组时，就不会只留下一个找不到调用的结果。

## 记录与下一轮输入各有职责

src/journal.rs 只有一个很小的 Journal。`record` 把事件写成一行 JSON，再 flush；create_new 则避免意外覆盖已有记录。每次 run 在工作区的 .crabloop 下创建独立文件，记录请求、成功解析的原始响应、工具开始/结束和停止；HTTP/协议失败记录 error，不保存失败 HTTP body。

这份记录用于回看发生过什么。ContextState::input() 则是一种选择：先放原任务，再放可选摘要，最后展开保留的交互组。改变这份选择，不需要删除 JSONL。

先用下面的命令观察一次新运行；模型配置沿用 v1：

```bash
cargo run --locked -- --workspace playground/my-first-repair "读取 NOTES.md，告诉我已有的验证结果"
```

stderr 会显示 journal 路径。用文本编辑器打开它，找到 request 事件的 data.input，以及对应的 response。日志里没有程序添加的 Authorization，但你自己的任务和文件内容可能在里面，因此运行记录默认留在本机。

这些还不是 v5 的可恢复 sessions：这里没有 resume，也不承诺跨版本重放。现在只需要让历史可观察。

## 固定约束与可计算的预算

工作区根目录有 AGENTS.md 时，我们把它加入 instructions。它与原任务都保留在固定位置，不交给未来摘要替换。文件不存在就继续；非 UTF-8 或超过 8192 字节则报错，让使用者缩短约定。

预算暂时用序列化请求的字节数。`request_bytes` 计算整个 body，包含 tools、instructions 和推理项，不只计算聊天文字。这个量能精确复现，但不是模型 token 数。

v2 默认 hard 为 49152 字节，soft 为它的四分之三。CRABLOOP_CONTEXT_BYTES 可以覆盖，最小 4096。soft 先为下一章留一个整理触发点；这一章仍只在 hard 不够时明确停止。

亲手做一个小实验：保存完整 input，然后只保留最近两组，再构造 input。原任务和 AGENTS 仍在，调用与结果也完整，但早先的失败测试可能已经消失。运行 `cargo test --locked --test context` 观察分组和预算检查；第 08 章会把这种损失与摘要方案放在同一份记录上比较。

组完整，只能保证协议结构成立。重要事实能否保留，还需要下一步的摘要与验证。

代码书签：step-06-context；[开发记录](../../docs/devlog/06-context-projection.md)。
