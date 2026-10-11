# 05：让四个工具一起做完一件事

到上一章为止，我们已经看见过读文件、编辑和测试，也看见过一轮响应如何进入下一轮。现在让它们一起完成一个小任务。先别看模型最后怎么说，我们先为“修好了”准备一个能在程序之外检查的事实。

本章命令都在自己的练习工程中执行。先从 step-04 状态应用本章增量，得到 fixture、verifier 和整理后的接口：

```bash
# 当前目录：练习工程根目录，包含 Cargo.toml。
git apply --check "$CRABLOOP_SOURCE/tutorial/v1/patches/05-v1.patch"
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/05-v1.patch"
cargo test --locked
```

CRABLOOP_SOURCE 指向参考仓库，按[增量入口](patches/README.md)设置；运行代码仍来自练习工程。已经应用过 05 时直接继续，不重复应用。

## 一个足够小、确实有错的函数

fixtures/buggy-rust/lib.rs 里有 `last_index(len)`。它应当在长度为零时返回 None，在长度非零时返回最后一个索引。初始版本却直接返回 Some(len)。长度与索引很接近，正适合观察模型究竟有没有运行测试。

在练习工程根目录准备一个待修复副本，选一个尚不存在的目录：

```bash
mkdir -p playground
cp -R fixtures/buggy-rust playground/my-first-repair
cd playground/my-first-repair
rustc --edition=2024 --test tests.rs -o checks
./checks
```

预期看到三个测试失败。执行 `cd ../..` 返回练习工程根目录，再运行 `bash scripts/verify-playground.sh playground/my-first-repair`，独立验收也应失败。后者的测试留在 harness 仓库里，检查 0、1、10 和 usize::MAX，避免把“模型改过测试”误当成修复。

## 给模型一个目标，把动作留给循环

按第 01 章配置模型与密钥之后运行：

```bash
cargo run --locked -- --workspace playground/my-first-repair \
  "阅读 README.md，修复 last_index。先运行原有测试，再修改，再测试。保留函数签名和 tests.rs，写 NOTES.md 记录实际验证。"
```

stderr 中会出现 turn 和 tool 记录；stdout 只留下最终回答。顺着记录看：模型先提出了什么，工具真实返回了什么，下一次动作是否反映了这个结果。模型可能合并几项调用，也可能多读一次文件，这些差异不影响任务是否完成。

完成之后再独立执行：

```bash
bash scripts/verify-playground.sh playground/my-first-repair
```

退出码为零说明这组独立用例通过。还要检查 tests.rs 没有变化，并读一遍 NOTES.md：里面的验证结论是否真的来自工具输出。模型的 Final 只说明它结束了这次回答；修复成立来自文件和测试。

本次真实运行中，DeepSeek V4.1 Flash 和备选 gpt-6-luna 都完成了“先失败、修复、再通过”，独立四项测试通过，原测试保持不变。[验收记录](../../evidence/v1/acceptance.md) 保存了模型、时间、差异与结果。时延只是这次运行的观察，不能由一条样本推断稳定性能。

## 把同一套循环用在资料上

四个工具并不只会修代码。fixtures/local-report/inbox 有三份小文档，包含负责人和状态；任务是读出事实，生成 report.json 和带来源的 SUMMARY.md。模型仍然要读、整理、写入和核对，只是最终产物换成了报告。

保持当前目录为练习工程根目录，沿用第 01 章配置好的进程环境，再准备一个新副本：

```bash
cp -R fixtures/local-report playground/my-first-report
cargo run --locked -- --workspace playground/my-first-report \
  "按 README.md 阅读全部 inbox 文档，生成 report.json 和带来源的 SUMMARY.md，保留原始资料，核对事实后结束。"
python3 scripts/verify-report.py playground/my-first-report
```

这些文件和命令都包含在 05 增量中。verify-report.py 只核对产物，行动循环仍是同一个 Rust 程序。这样可以在同一个 v1 练习工程里完成修复与资料整理。

这次实验还暴露了验收代码自己的问题：模型给 counts 多写了一个正确的 total=3，我们最初按整个对象严格相等比较，把正确报告判错了。修正后只检查任务要求的事实，允许正确的附加字段；同一份产物重新验收通过，没有重新调用模型。原失败和复核都保留。写 verifier 时也要问：这条限制究竟来自用户目标，还是只是我们恰好想到的输出形状？

## 把阶段接口收束到最终 v1

本章完整改动在 [05-v1.patch](patches/05-v1.patch)。它以 step-04-loop 为起点：加入练习与独立验收，同时去掉两层已经没有必要的包装。迁移关系如下：

| 阶段写法 | v1 最终写法 | 状态交给谁 |
|---|---|---|
| ModelClient 保存端点、Key、模型等配置 | run 持有 reqwest::Client，函数借用 Config | Config 是配置的单一来源 |
| client.request(...) | model::request(&config, ...) | model 模块构造协议 body |
| client.respond(request).await | model::respond(&client, &config, request).await | 复用同一个 HTTP client |
| execute 返回 ToolResult，再 as_input() | execute 返回 JSON，loop 包装 function_call_output | call_id 关联由 loop 负责 |

最后一项对应 run 内这段完整替换：

```rust
for call in reply.calls {
    let result = tools::execute(&call, &config.workspace, &config.limits).await;
    eprintln!("tool {}", json!({
        "call_id": call.call_id, "name": call.name, "result": result
    }));
    input.push(json!({
        "type": "function_call_output",
        "call_id": call.call_id,
        "output": result.to_string()
    }));
}
```

源码顶层的导入、函数签名以及测试调用一起在 05 diff 中更新。应用后运行 `cargo test --locked`，会看到 29 项 Rust 测试通过。第 01–04 章保留当时的写法，迁移从这里明确发生。

随后可应用 [06-readable.patch](patches/06-readable.patch)，把 schema、请求字段和工具结果展开。这份教学排版副本是 352 代码行、357 物理行，行为与原始 v1 相同；再次运行上述测试和 `cargo fmt --check`。它让读者在代码里更容易找到模型看见的协议字段。

## 在 v1 停一下，再看下一步

原始 v1 标签的核心是 314 行非空、非独立 // 注释的 Rust，物理行数 319。为了让状态归属更清楚，我们删去了重复存储配置的 ModelClient，让 run 持有 reqwest::Client，model::request/respond 接收借用的 Config；工具返回 JSON，loop 负责包装 function_call_output。这也是前几章旧接口在最终 v1 中发生的小幅整理，行为没有改变。

格式统一为 100 列、rustfmt 的 Max 小表达式排版；语义简化与格式变化都记录在开发日志里。300 行是提醒我们只做核心问题，350–360 行同样可以接受，数字不值得牺牲可读性。

现在试着只看一条工具轨迹，解释 call_id 从哪里来、谁执行动作、谁把结果加入历史。如果你能顺着一次失败走到下一次行动，v1 的主要学习目标就达到了。

还剩一个问题：每轮把历史全部重发，文件和终端输出会不断累积。下一版保留这四个工具，开始改变“下一轮看见哪些历史”。

对应书签：v1。实际经过见[开发记录 05](../../docs/devlog/05-v1-coding-task.md)。
