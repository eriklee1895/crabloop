# 观察 CrabLoop 的执行流程

Agent 报告“完成了”，但你想知道它看过哪些文件、怎样处理失败，以及下一次模型请求为什么变了。这时可以把一次任务停在几个关键位置，沿着真实数据走一遍。

这篇指南放在 main，以下路径和函数都以 main 为准。先用固定响应测试认识模型—工具循环，再观察真实 HTTP 请求；v1 的学习代码继续保持原样。

## 先选一组够用的工具

| 工具 | 用来回答什么问题 | 建议从哪里开始 |
|---|---|---|
| RustRover，或 VS Code + rust-analyzer + CodeLLDB | 程序保存了什么状态，哪个分支决定继续或停止？ | 调试一个现有测试 |
| Proxyman | 客户端实际发出了什么 HTTP 请求，服务返回了什么？ | macOS 上查看 Request、Response 和 JSON |
| mitmproxy / mitmweb / mitmdump | 怎样用脚本处理流量，保存可重复分析的样本？ | 需要自动化时再加入 |

如果已有顺手的编辑器，先沿用它。RustRover 可直接从测试方法启动 Debug；VS Code 的 rust-analyzer 提供 Run / Debug 入口，macOS 可搭配 CodeLLDB。[RustRover 测试调试](https://www.jetbrains.com/help/rust/performing-tests.html)、[VS Code Rust 调试](https://code.visualstudio.com/docs/languages/rust)

Proxyman 的 JSON、Tree View 和 Raw 视图适合并排阅读请求与响应；需要程序化处理时，mitmproxy 提供 Python 脚本能力，mitmweb 是网页界面，mitmdump 是命令行入口。[Proxyman 消息查看器](https://docs.proxyman.com/basic-features/request-response-viewer)、[mitmproxy 工具说明](https://docs.mitmproxy.org/stable/)

## 用固定响应走完一次错误修正

在仓库根目录确认当前分支为 main，先普通运行这个测试：

```bash
git branch --show-current
cargo test --locked --test loop tool_error_then_corrected_call_receives_actionable_feedback \
  -- --exact --nocapture --test-threads=1
```

测试在临时目录创建内容为 `OLD` 的 lib.rs，再安排三次响应：第一次 edit_file 查找 `missing`，第二次查找 `OLD` 并替换为 `NEW`，第三次给出最终回答。响应由本地 HTTP stub 提供，文件操作仍是真实执行；不需要模型 Key，也不调用外部模型。

打开 [tests/loop.rs](../tests/loop.rs)，找到同名函数。RustRover 中右键选择 Debug，或点击旁边的运行图标；VS Code 中点击函数上方的 Debug。使用普通 debug 构建，先不要选择 release。

在 [src/lib.rs](../src/lib.rs) 的 `Session::turn` 中设置下面这些断点。以语句定位，源码行号随演进可能变化；停在某行表示该行通常还没有执行。

| 停在哪里 | 展开哪些变量 | 要弄清楚什么 |
|---|---|---|
| `self.journal.record("request", &request)?` | `input`、`request` | 模型将看到哪些历史和工具定义？ |
| `self.journal.record("response", &reply.raw)?` | `reply.raw`、`reply.calls`、`reply.is_final` | 这轮响应包含文字、调用，还是两者都有？ |
| `progress(Progress::ToolStart(&call))` | `call.name`、`call.arguments`、`call.call_id` | 即将执行哪个动作？ |
| 写入 `tool_finished` 记录的位置 | `result`、`call.call_id` | 工具实际成功还是失败？ |
| `if final_answer` | `self.state`、`report` | 完整交互组已经保存了吗，为什么结束？ |

第一次调用中，`old_text` 是 `missing`。执行之后，result 的 ok 为 false，临时文件仍是 OLD。继续到下一轮请求，找到失败调用和 function_call_output，检查二者使用同一个 call_id。第二次编辑成功后，文件变成 NEW；第三轮没有工具调用，循环以 Final 结束。测试末尾还独立检查文件和回传的错误内容。

观察整个回路时，用 Continue 跳到下一处断点。只在想看某个具体函数时用 Step Into，例如进入 [src/tools.rs](../src/tools.rs) 的 `execute_inner`，观察唯一匹配检查。Rust 的 async 单步可能进入 Future、Tokio 或 HTTP 库；先在业务语句之间观察状态，更容易保留完整思路。

变量面板可先展开 String、Vec 和 struct 字段，再看 serde_json::Value 的内容。复杂 Rust 表达式能否在 Watch 中求值取决于调试器；遇到无法求值时，回到局部变量和现有 JSON 轨迹。

HTTP 超时是 60 秒，Bash 超时是 30 秒；本地 stub 在读取请求时也有 3 秒超时。暂停期间外部时间仍在经过，长时间停在进行中的网络读写或子进程操作里，恢复后可能看到超时。第一次练习尽量停在请求完成后、工具调用前后，暂不单步超时测试。

## 再看真实模型和本地记录

读懂固定测试后，可以在 Cargo 运行配置中使用下面的命令，工作目录设为仓库根目录，沿用该目录的 .env：

```text
run --locked -- -p "读取 Cargo.toml，列出直接依赖的名称，不修改文件。"
```

保留相同断点，观察这次模型自己选择了什么动作。这个任务是否调用 read_file 或 bash、用了几轮，需要以本次实际结果为准。

commentary 是面向用户的中间说明，final_answer 是完成后的回答，reasoning 则是另一类协议项。当前终端显示工具进度和最终答案，尚未独立显示 commentary；它仍保存在完整 output 和 JSONL 中。最终文本选择不会删除这些回放项。可读 reasoning summary 与加密 reasoning 也需区分，后者不作为展示文本。

main 会在工作区 .crabloop 中保存 JSONL。终端打印本次 journal 路径后，只打开这次运行的文件，沿事件顺序阅读：

```text
user → request → response → tool_started → tool_finished → ... → stop
```

request 保存发送的 JSON body，response 保存成功解析的原始响应；tool_started 和 tool_finished 用 call_id 关联。下一次 request 中的 function_call_output 应与刚才的工具结果相符。若触发压缩，还会出现 summary_request、summary_response 和 compaction；摘要请求与主模型请求分别记录，HTTP 请求数可能多于主循环轮数。

记录层和网络层各有边界。Authorization 在发送 HTTP 时加入，不在 request 的 JSON body 中；但 journal 仍包含任务、文件内容和工具输出。HTTP 错误会先经过 error_for_status，因此错误响应正文不会变成成功的 response 记录。这时抓包有助于查看服务实际返回的状态和错误正文。

## 用 Proxyman 观察一次 HTTP 通信

第一次抓包先只观察，不同时开启修改响应的规则。让 IDE、journal 与 HTTP 消息对应起来之后，再做故障实验。

1. 在 Proxyman 中确认本地监听地址与端口。官方示例使用 `http://localhost:9090`；如果你修改过端口，以当前设置为准。
2. 按 [macOS 证书步骤](https://docs.proxyman.com/debug-devices/macos)安装并信任本机 Proxyman CA。
3. 只对本次使用的模型服务域名开启 [SSL Proxying](https://docs.proxyman.com/basic-features/ssl-proxying)，例如 api.deepseek.com 或自己的兼容网关域名。
4. 给启动 CrabLoop 的进程提供 HTTPS_PROXY，再执行一个小的只读任务。

```bash
# 仓库根目录；9090 替换为 Proxyman 当前监听端口。
HTTPS_PROXY=http://127.0.0.1:9090 \
  cargo run --locked -- -p "读取 Cargo.toml，列出直接依赖的名称，不修改文件。"
```

这一步要使用进程环境变量。CrabLoop 的 .env 是配置数据，CLI 只读取所需配置；把 HTTPS_PROXY 只写进 .env，不会自动把它设置给 reqwest。IDE 调试时，也要在运行配置的 Environment variables 中设置，而不是只在另一个终端 export。

当前 [Cargo.toml](../Cargo.toml) 的 reqwest 0.13.5 关闭 default features，仅开启 json、rustls，没有开启读取 macOS 系统代理设置的 system-proxy。它仍会读取代理环境变量，因此显式 HTTPS_PROXY 更可靠。NO_PROXY/no_proxy 若匹配目标域名，会绕过代理。[reqwest 0.13.5 代理规则](https://docs.rs/reqwest/0.13.5/reqwest/#proxies)、[hyper-util 代理匹配](https://docs.rs/hyper-util/0.1.21/hyper_util/client/proxy/matcher/struct.Matcher.html)

这份 reqwest 构建使用 rustls-platform-verifier，默认验证器读取平台信任；macOS 的 CA 信任配置是 HTTPS 解密的另一半。保持客户端证书校验，遇到证书错误时先核对 CA 是否已信任和目标域名是否开启 SSL Proxying，不在 Harness 中关闭 TLS 验证。[固定版本客户端源码](https://docs.rs/reqwest/0.13.5/src/reqwest/async_impl/client.rs.html)

有流量后，先按模型服务域名和 `/responses` 筛选，再检查这五件事：

- 请求的实际 URL、模型名，以及是否走了预期网关。
- 请求 body 中的 instructions、tools 和 input。
- HTTP 状态、响应 Content-Type，以及 JSON 内的 status。
- output 中的 function_call、call_id、arguments、message 和 phase。
- 下一次请求是否包含原调用及对应 function_call_output。

HTTP 状态和响应内的 status 是两个判断。一次 HTTP 200 也可能带来不满足 completed 契约的响应。抓包能说明服务返回了什么；程序是否执行了动作，要继续对照 tool_finished 和文件状态。

没有流量时，先检查启动进程的代理变量、监听端口和 NO_PROXY。只有 CONNECT 或看不到明文 body 时，检查 CA 信任与 SSL Proxying。浏览器能抓到流量，也不能单独证明 Rust 进程已经走同一代理。

当前 main 接收完整 JSON，基础 SSE 仍在规划中。抓包只能看到客户端与所连接服务之间的通信，无法展开网关内部的上游调用或模型内部推理；reasoning.encrypted_content 保持为不透明协议数据。

## 对照和故障实验

用一条 function_call 的 call_id，把三处证据连起来：响应里的调用、本地工具完成记录、下一次请求里的回传。再检查文件或测试结果，才能判断任务是否成功。

掌握观察后，可以在可丢弃工作区中尝试 Proxyman 的 [Map Local](https://docs.proxyman.com/advanced-features/map-local) 或 [Breakpoint](https://docs.proxyman.com/advanced-features/breakpoint)，替换一个 HTTP 状态或响应正文，观察客户端的错误路径。代理规则应只匹配这次实验的请求，结束后关闭规则。网络断点暂停也会消耗客户端超时，带暂停的耗时不作为模型性能数据。

需要长期自动化同类实验时，再用 mitmproxy 的 Python 脚本或 mitmdump；已有固定测试继续负责确定性的协议检查。保存 HAR、代理会话或 journal 时，先移除 Authorization、Cookie 和私有任务内容；公开复盘保留必要字段和事实，不提交原始会话或凭证。
