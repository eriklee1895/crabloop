# 04：把结果送回去，循环就开始了

到这里，模型协议和四个工具都已经能够独立工作。我们还没有得到完整的 agent，因为执行工具以后，结果没有自动回到模型手里。现在来接上这条回路。

从 step-03-bash 开始，在 src/lib.rs 增加 run 函数，让它持有这一次任务的历史。src/main.rs 随之变成一个很薄的入口：读取配置，调用 run，打印最后的报告。

## 历史由谁持有

先创建 HTTP 客户端和工具 schemas，再用原始用户任务建立 input：

```rust
let mut input = vec![json!({"role":"user", "content":config.task})];
```

这份 Vec<Value> 由 run 持有。每次请求发送的是它当时的内容；每次模型与工具带回新的事实，我们就在这里追加。这让你可以顺着一个变量理解整次任务，不需要先引入会话数据库或复杂框架。

在限定轮数的 for 循环中，构造请求、检查大小、等待响应：

```rust
let request = client.request(INSTRUCTIONS, &input, &tools);
let reply = client.respond(request).await?;
input.extend(reply.output);
```

实际代码在发送前比较整个请求 JSON 的字节大小，超限就明确停止。这个数字是一个输入体积保护，不是准确 token 数；工具 schemas、指令和可回传推理项都包含在内。

最后一行把模型的完整输出移交给历史。只有保存了原调用，后面追加的结果才能找到对应的来源。我们也继续保留 reasoning 和 phase 等字段。

## 一轮里可能有几个动作

模型可能同时提出读取 lib.rs 和 tests.rs。我们的第一版逐个执行它们：

```rust
for call in reply.calls {
    let result = tools::execute(&call, &config.workspace, &config.limits).await;
    input.push(result.as_input());
}
```

注意这段循环位于下一次模型请求之前。两个调用分别产生两个结果，全部进入历史后，再继续请求模型。工具内部失败也会产生结果，所以缺少文件、找不到旧文本、测试非零退出，都能成为下一轮的输入。

试着运行 cargo test --locked --test loop two_calls_then_final。测试使用真实文件工具，检查第二次 HTTP 请求里两个结果各自携带哪个 call_id，以及它们是否读到了正确文件。你可以暂时交换两个结果的 id，观察这个检查能否发现错误，然后恢复代码。

## 循环必须知道怎样停下来

我们用 StopReason 区分三种正常控制结果：Final、MaxTurns、ContextLimit。HTTP 或协议错误通过 Result 返回调用方。RunReport 同时保存最终文字与实际主模型调用次数。

如果响应里还有工具调用，就先执行它们。只有没有调用、并且模型给出了最终回答时，才把原因设为 Final。只有 commentary 的响应会进入历史，再继续一轮；它也消耗轮数，因此不会无限循环。

让 fake server 连续返回两次 commentary，并把 max_turns 设为 2。第三次请求不应该发生。再把请求体积上限设为 1，连第一次网络请求都不应该发出。这两个实验分别观察迭代预算和输入预算，它们解决不同的问题。

## 错误是下一步的材料

另一项测试安排模型先提出错误的 edit_file，再提出正确的匹配。真正执行的第一次编辑会失败，文件保持原样；它的错误说明进入下一次请求，第二次编辑才成功。

这个脚本化模型帮助我们稳定检查反馈通路。它并不能证明真实模型每次都能自我纠错。接下来的综合任务会换成真正的模型，让它在一个带 bug 的小项目里自己决定下一步。

最后留意 Final 的含义：它表示模型结束了回答。一个模型完全可能在测试失败后仍说“已经修好”。因此 CLI 的正常退出和项目测试通过要分别检查；下一章会引入位于练习目录之外的独立验收程序。

运行 cargo test --locked --test loop，再运行完整的 cargo test --locked。此时你应该能从 input 的第一次创建，一直追到某个明确的结束原因，解释每次模型究竟看到了什么。

对应代码书签：step-04-loop。源码见 [lib.rs](../../src/lib.rs)，验证见[开发记录](../../docs/devlog/04-agent-loop.md)。
