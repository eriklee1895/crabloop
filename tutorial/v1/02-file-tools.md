# 02：让一个工具调用真正读到文件

上一章结束时，我们已经能识别 read_file 这样的请求。现在把模型放在一边，先完成被调用的那段 Rust 代码。给它一个路径，看看它是否真的能读出文件；给它错误的参数，再看看文件是否保持原样。

从 step-01-responses 的代码开始，在 src/lib.rs 加入 pub mod tools，并创建 src/tools.rs。我们将在这个文件里逐个增加三个文件工具。对应的练习与验收放在 tests/tools.rs，使用临时目录，避免实验改到真实项目。

## 先把两层 JSON 分开

ToolCall.arguments 是字符串。执行入口先把它解析为 serde_json::Value，然后提取 path：

```rust
let args: Value = serde_json::from_str(&call.arguments)?;
let arg = |key: &str| {
    args[key].as_str().with_context(|| format!("{key} must be a string"))
};
```

这个小闭包会在后面的三个工具里反复使用。它返回借用的 str，原始字符串仍由 args 持有；执行函数还在运行时，args 也一直存在。遇到 path 是数字或根本不存在的情况，闭包返回错误，文件操作尚未开始。

我们约定文件路径相对于工作目录，并拒绝绝对路径与 ..。这让工具接口的含义明确：path 指向当前项目中的哪个位置。它不是操作系统沙箱，符号链接和后面的 Bash 仍需要另外的执行边界。

## 读文件时就控制大小

最直接的办法是 read_to_string。但这一步有一个值得亲手试的边界：一个有效的中文文件，在字节限制处截断，末尾可能只剩半个字符。

假设文本是“你你好”，一共 9 个 UTF-8 字节。先取 8 个字节，再立即转成字符串，得到的错误会是无效 UTF-8；文件本身却没有编码问题。我们在测试中真实复现了这个现象。

因此先读取有上限的字节，再检查大小，最后解码：

```rust
let mut bytes = Vec::new();
fs::File::open(&path)?
    .take(limits.file_bytes as u64 + 1)
    .read_to_end(&mut bytes)?;
ensure!(bytes.len() <= limits.file_bytes, "file exceeds size limit");
let text = String::from_utf8(bytes)?;
```

多读的那一个字节用于判断是否超限。交给模型的文本还有更小的输出上限；clip_utf8 会退回到合法字符边界，同时设置独立的 truncated 标记。模型既能看到保留下来的内容，也知道这份内容不完整。

运行 cargo test --locked --test tools read_truncation_is_utf8_safe_and_non_utf8_is_an_error，再运行 oversized_utf8_reports_size_instead_of_encoding，比较“文件太大”和“文件不是有效 UTF-8”这两种结果。

## 从写入走到精确修改

write_file 接受完整 content，用于创建或覆盖文件。先检查长度，再写入；父目录不存在时返回错误。它容易理解，但为了只改一小处而重写整个文件，会给模型增加不必要的工作。

edit_file 因此接受 old_text 与 new_text。我们希望它只改明确的一处。如果原文找不到，应该让模型重新读取文件；如果出现多处，应该让模型提供更具体的片段。

在读取到的 text 上找到第一个和最后一个匹配位置：

```rust
let start = text.find(old).context("old_text did not match; read the file again")?;
ensure!(text.rfind(old) == Some(start), "old_text must have a unique match");
text.replace_range(start..start + old.len(), arg("new_text")?);
```

进入这段代码之前，先拒绝空的 old_text。修改仍然发生在内存中；新内容的大小通过检查以后，才写回磁盘。因此空匹配、零匹配、多匹配和结果超限这些验证失败，都不会改变原文件。

试着让文件包含两次 same，再用 same 作为 old_text。看看错误内容，然后读一遍磁盘上的文件。仅仅检查 ok: false 还不够；我们也要确认错误路径没有先把文件改坏。这个工具暂时使用普通文件写入，写入期间的 I/O 故障不提供原子回滚保证。

## 给下一轮模型留下可以使用的结果

工具内部用 Result 表达失败。外层 execute 把可修正的错误变成带 ok: false 的 JSON，随后 ToolResult::as_input 生成 Responses 的 function_call_output。这里继续使用原始 call_id，而 output 字段装的是序列化后的 JSON 字符串。

工具描述也在本章加入。schemas 声明工具名、用途和参数类型，让模型有机会正确提出请求；执行函数仍要验证真实参数。这两层相互配合，不能因为 schema 写了 string，就默认收到的每一份输入都有效。

运行 cargo test --locked --test tools。正常读写、唯一替换、错误参数、路径约定和文本边界通过后，文件工具已经能独立工作。下一步我们要接触一种不同的资源：启动后会持续运行、产生输出、也可能迟迟不退出的子进程。

对应代码书签：step-02-file-tools。实现见 [tools.rs](../../src/tools.rs)，实验经过见[开发记录](../../docs/devlog/02-file-tools.md)。
