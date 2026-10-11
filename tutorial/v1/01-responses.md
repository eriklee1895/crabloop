# 01：把第一条请求送出去

上一章里，我们画出了任务、模型和工具之间的来回。现在先接上最短的一段：程序发出请求，等待响应，把其中的文字打印出来。

这一章完成时，CrabLoop 还不会读取你的代码。我们先确认它能够正确收发消息，再给它增加动作。

## 从空目录发出第一次请求

如果你要亲手从零写，先在参考仓库之外建一个练习目录：

```bash
cargo new crabloop-lab --edition 2024
cd crabloop-lab
```

将 Cargo.toml 的 dependencies 部分替换成下面这些基础库。后续章节会用到 serde 和 tokio 的进程能力，所以这里一并声明；暂时不需要理解所有 feature。

```toml
[dependencies]
anyhow = "=1.0.104"
reqwest = { version = "=0.13.5", default-features = false, features = ["json", "rustls"] }
serde = { version = "=1.0.229", features = ["derive"] }
serde_json = "=1.0.152"
tokio = { version = "=1.53.2", features = ["macros", "rt-multi-thread", "process", "io-util", "time"] }
```

先把 src/main.rs 完整替换为这个只做一次往返的程序：

```rust
use serde_json::{Value, json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let key = std::env::var("OPENAI_API_KEY")?;
    let model = std::env::var("CRABLOOP_MODEL")?;
    let base = std::env::var("OPENAI_BASE_URL")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let response: Value = client
        .post(format!("{}/responses", base.trim_end_matches('/')))
        .bearer_auth(key)
        .json(&json!({
            "model": model, "input": "只回复 CRABLOOP_PROTOCOL_OK",
            "store": false, "max_output_tokens": 2048
        }))
        .send().await?
        .error_for_status()?
        .json().await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
```

现在运行 `cargo check`，它会生成 Cargo.lock；后续使用 `cargo check --locked`。配置本章后面列出的三个环境变量，再运行 `cargo run --locked`。这是一个教学小探针，打印整个响应，方便找到 output、message 和文本的位置；它还没有 CLI 任务、工具、循环，也不属于最终实现的另一份运行时代码。

注意两个问号的含义：HTTP 请求可以失败，JSON 解码也可以失败。`Result` 把这些失败交回入口，`async` 则允许等待网络时让出执行权。先让这段代码工作，再添加类型与校验，比一开始就面对一个完整 agent 更容易看清控制流。

下面回到参考仓库的 `step-01-responses`。可以用 `git show step-01-responses:src/model.rs` 对照这一章的完整文件；亲手练习时，把单次 HTTP 调用移入 model.rs，再逐项添加下面解释的返回类型与验证。后续章节通过同样的书签保留每一步可运行的参考状态。

## 先准备能运行的入口

以下在参考仓库根目录运行；教学探针则保留在自己的练习目录。Cargo.toml 固定了 HTTP、JSON 和异步运行需要的基础库；Cargo.lock 记录解析后的完整依赖版本。运行 cargo test --locked --test protocol，可以先在本地检查协议，不需要真实密钥。

我们把程序入口留在 src/main.rs，把模型交互放进 src/model.rs。入口拿到 Config 后，创建 ModelClient，发出一次请求：

```rust
let client = ModelClient::new(
    &config.endpoint, &config.model, &config.api_key, &config.limits,
)?;
```

这里的问号会把构建失败继续交给调用方。HTTP 客户端还带着请求超时；对一个调用外部服务的程序来说，“一直等下去”也需要有结束的办法。

Config 暂时只从命令行和进程环境读取设置。它接收工作目录和任务，确认目录存在，并要求模型名和密钥非空。你在本地保存的 .env 不会被程序自动执行；先把需要的变量导出到终端，或者使用后面真实验收的运行脚本。

本项目优先使用 DeepSeek V4.1 Flash，对应 API 模型名 deepseek-flash；gpt-6-luna 经 TeamoRouter 作为测试备选。选择 DeepSeek 时，将它的密钥放入当前进程的 OPENAI_API_KEY，OPENAI_BASE_URL 设为 https://api.deepseek.com，CRABLOOP_MODEL 设为 deepseek-flash。OPENAI_BASE_URL 是 URL 前缀，程序会补上 /responses；CRABLOOP_API_URL 则可以直接指定完整端点，优先级更高。

```bash
export CRABLOOP_MODEL=deepseek-flash
export OPENAI_BASE_URL=https://api.deepseek.com
# 在当前终端配置 OPENAI_API_KEY，然后运行：
cargo run --locked -- --workspace . "只回复 CRABLOOP_PROTOCOL_OK"
```

密钥属于本地配置，.env 已被 Git 忽略。仓库里的示例只保留变量名。

## 请求中到底装了什么

打开 ModelClient::request，可以看见请求 body 的构造。model 指定模型，instructions 给出这次程序的约定，input 放入任务与历史，tools 声明可以调用的工具。当前入口传入空 tools，因此真实模型还没有我们提供的文件工具可用。

我们设置 store: false，并由客户端维护 input。等进入循环时，你能直接查看下一轮究竟发送了哪些项；到了 v2，也能亲手改变这份输入。max_output_tokens 限制模型输出，include 请求可回传的推理信息。不同 provider 的字段支持要以实际接口和测试为准。

一次正常的文字响应会在 output 数组中包含 assistant message。文本位于 message 的 content 项里，类型为 output_text。这里读的是 HTTP JSON，所以需要亲手走到这层结构；SDK 示例中常见的 output_text 快捷属性不等于 HTTP 响应必然有同名顶层字段。

## 先认识工具调用这份数据

真实工具要到后续章节才接入。我们现在借助本地样本，先看另一类 output item：

```json
{
  "type": "function_call",
  "id": "fc_call_1",
  "call_id": "call_1",
  "name": "read_file",
  "arguments": "{\"path\":\"lib.rs\"}"
}
```

注意两个 id。id 标识这个输出项，call_id 才用于关联工具结果。arguments 在这里是一个字符串，字符串内部装着 JSON。模型协议解码和工具参数解码因此是两件事：这一章识别调用，下一章才验证 path 是否可用。

ToolCall 用一个小 struct 保存这三个需要执行的字段：call_id、name、arguments。与此同时，ModelReply 仍保留完整的 output 项。推理信息和 assistant phase 也在其中；我们不理解某项不透明内容，并不意味着可以在下一轮丢掉它。

## 在错误发生时停对地方

这一章的测试会故意返回 incomplete。它表示响应没有完整生成，里面即使出现了工具调用，也不应该马上执行。程序先检查响应状态，再解析动作。

另一份样本只有 commentary，也就是中间说明。它可以出现在历史里，却不能被当作任务已经结束。真正的最终回答和中间叙述需要区分；我们把这个判断整理成 ModelReply.is_final，后面的 loop 会使用它。

你还可以运行 cargo test --locked --test protocol incomplete_and_missing_or_duplicate_call_ids_are_rejected，观察本地服务器返回缺少 id、重复 id 和不完整响应时，程序是否给出错误。这里执行的仍然是真正的 HTTP 请求与解析，只是服务端返回了我们事先写好的 JSON，便于稳定制造故障。

本地协议测试证明程序能处理这些约定；真实 provider 的成功请求则是另一层证据。开发记录会分别记录两者。等你能解释请求里的 input、响应里的 output，以及 call_id 的用途，我们就可以给 read_file 接上一段真正读取磁盘的 Rust 代码了。

对应代码书签：step-01-responses。源码见 [model.rs](../../src/model.rs)，本步验证见[开发记录](../../docs/devlog/01-responses.md)。
