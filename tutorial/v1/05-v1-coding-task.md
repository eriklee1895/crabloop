# 05：让四个工具一起做完一件事

到上一章为止，我们已经看见过读文件、编辑和测试，也看见过一轮响应如何进入下一轮。现在让它们一起完成一个小任务。先别看模型最后怎么说，我们先为“修好了”准备一个能在程序之外检查的事实。

本章命令都在自己的练习工程中执行。先从 step-04 状态应用综合任务和最终入口两个增量，得到 fixture、verifier 与当前 v1：

```bash
# 当前目录：练习工程根目录，包含 Cargo.toml。
git apply --check "$CRABLOOP_SOURCE/tutorial/v1/patches/05-v1.patch"
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/05-v1.patch"
git apply --check "$CRABLOOP_SOURCE/tutorial/v1/patches/06-v1-entry.patch"
git apply "$CRABLOOP_SOURCE/tutorial/v1/patches/06-v1-entry.patch"
cargo test --locked
```

CRABLOOP_SOURCE 指向参考仓库，按[增量入口](patches/README.md)设置；运行代码仍来自练习工程。已经应用过的增量不重复应用。完成 06 后，全部 src 文件与 `milestone/v1` 一致，后面的实验使用这一份 v1。

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

当前 v1 从启动目录读取 .env，也接受进程环境。本章从练习副本启动二进制，密钥可通过进程环境提供；使用 DeepSeek 时设置 DEEPSEEK_API_KEY，沿用第 01 章的 OPENAI_API_KEY 等变量时，先执行 `export CRABLOOP_PROVIDER=openai`。若使用 .env，应放在相应练习副本目录，格式见练习工程的 .env.example。

v1 把当前目录作为工作区，任务统一从 stdin 读入。直接启动时输入一行任务；这里用管道提交完整任务，便于复现实验。参数和配置说明放在 README 与 .env.example，入口代码集中展示读取任务和准备 Config 的过程。先在练习工程构建，再从待修复副本启动同一个二进制；括号里的目录切换在结束后自动返回：

```bash
cargo build --locked
crabloop_bin="$PWD/target/debug/crabloop"
(
  cd playground/my-first-repair
  printf '%s\n' "阅读 README.md，修复 last_index。先运行原有测试，再修改，再测试。保留函数签名和 tests.rs，写 NOTES.md 记录实际验证。" | "$crabloop_bin"
)
```

stderr 中会出现 turn 和 tool 记录；stdout 只留下最终回答。顺着记录看：模型先提出了什么，工具真实返回了什么，下一次动作是否反映了这个结果。模型可能合并几项调用，也可能多读一次文件，这些差异不影响任务是否完成。

完成之后再独立执行：

```bash
bash scripts/verify-playground.sh playground/my-first-repair
```

退出码为零说明这组独立用例通过。还要检查 tests.rs 没有变化，并读一遍 NOTES.md：里面的验证结论是否真的来自工具输出。模型的 Final 只说明它结束了这次回答；修复成立来自文件和测试。

已有真实运行中，历史步骤的 DeepSeek V4.1 Flash 和备选 gpt-6-luna 都完成了“先失败、修复、再通过”；当前 v1 也有 [DeepSeek 修复记录](../../evidence/v1/20261012T035314137932Z-coding-deepseek.json)，独立验证通过，原测试保持不变。[历史验收记录](../../evidence/v1/acceptance.md) 保留各次运行的模型、时间、差异与结果。时延只是单次观察，不能由一条样本推断稳定性能。

## 把同一套循环用在资料上

四个工具并不只会修代码。fixtures/local-report/inbox 有三份小文档，包含负责人和状态；任务是读出事实，生成 report.json 和带来源的 SUMMARY.md。模型仍然要读、整理、写入和核对，只是最终产物换成了报告。

保持当前目录为练习工程根目录，沿用上一节配置好的进程环境，再准备一个新副本：

```bash
cp -R fixtures/local-report playground/my-first-report
cargo build --locked
crabloop_bin="$PWD/target/debug/crabloop"
(
  cd playground/my-first-report
  printf '%s\n' "按 README.md 阅读全部 inbox 文档，生成 report.json 和带来源的 SUMMARY.md，保留原始资料，核对事实后结束。" | "$crabloop_bin"
)
python3 scripts/verify-report.py playground/my-first-report
```

这些文件和命令都包含在 05 增量中。verify-report.py 只核对产物，行动循环仍是同一个 Rust 程序。这样可以在同一个 v1 练习工程里完成修复与资料整理。

这次实验还暴露了验收代码自己的问题：模型给 counts 多写了一个正确的 total=3，我们最初按整个对象严格相等比较，把正确报告判错了。修正后只检查任务要求的事实，允许正确的附加字段；同一份产物重新验收通过，没有重新调用模型。原失败和复核都保留。写 verifier 时也要问：这条限制究竟来自用户目标，还是只是我们恰好想到的输出形状？

## 把阶段接口收束到最终 v1

接口整理在 [05-v1.patch](patches/05-v1.patch)。它以 step-04-loop 为起点：加入练习与独立验收，同时去掉两层已经没有必要的包装。迁移关系如下：

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

源码顶层的导入、函数签名以及测试调用一起在 05 diff 中更新。05 中间步骤有 29 项 Rust 测试，第 01–04 章保留当时的写法，迁移从这里明确发生。

本章开头接着应用的 [06-v1-entry.patch](patches/06-v1-entry.patch)展开 schema、请求字段和工具结果，再把启动职责集中到 src/cli.rs：工作区固定为 cwd，支持单任务输入和 .env 数据读取。最终测试集为 42 项 Rust；再运行 `cargo fmt --check` 和 `python3 scripts/count-lines.py --max 399`，检查格式与完整生产代码规模。

## 在 v1 停一下，再看下一步

v1 统一以 `milestone/v1` 为学习终点，全部生产 Rust 为 388 代码行、449 物理行。其中单任务输入与配置的 CLI 为 62 代码行，其余实现合计 326 行。run 持有 reqwest::Client 和任务历史，model::request/respond 接收借用的 Config；工具返回 JSON，loop 负责包装 function_call_output。

格式统一为 100 列、rustfmt 的 Max 小表达式排版。约 300 行约束核心范围，全部生产 Rust 严格 <400；保持可读排版，超预算时减少功能。

现在试着只看一条工具轨迹，解释 call_id 从哪里来、谁执行动作、谁把结果加入历史。如果你能顺着一次失败走到下一次行动，v1 的主要学习目标就达到了。

还剩一个问题：每轮把历史全部重发，文件和终端输出会不断累积。下一版保留这四个工具，开始改变“下一轮看见哪些历史”。

完整学习代码：`milestone/v1`。实际经过见[开发记录 05](../../docs/devlog/05-v1-coding-task.md)；历史标签保存当时的步骤和验收证据。
