# 03：让测试作为一个真实进程运行

现在三个文件工具可以独立读写磁盘了。为了判断改动有没有效果，我们还需要运行项目自己的测试。这一次，工具执行的对象会持续存在一段时间，也可能在我们等待时不断输出内容。

从 step-02-file-tools 开始，在 schemas 中加入 bash，参数只有 command。然后在 execute_inner 解析参数后增加一个分支，把命令交给单独的异步函数。文件工具和 Bash 仍通过同一个 execute 入口返回结果。

## 启动只是开始

我们明确启动 bash -c，并把工作目录交给 current_dir：

```rust
let mut child = tokio::process::Command::new("bash")
    .args(["-c", command])
    .current_dir(workspace)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true)
    .spawn()?;
```

stdin 关闭，避免一个等待输入的命令一直等着终端。stdout 与 stderr 改为管道，程序才能拿到测试报告和错误信息。spawn 返回 Child；此时命令可能还在运行，不能因为启动成功就报告测试通过。

先运行 cargo test --locked --test tools bash_reports_exit_and_stderr。测试命令向两个管道写入不同文本，最后以 7 退出。退出码和两份输出会被分别保存，ok 为 false。以后模型收到的正是这些执行事实。

## 为什么需要同时读两个管道

管道有容量。如果先等待进程结束，却不读取它的输出，进程可能正因为管道写满而无法继续。反过来，只读取 stdout 也不够，stderr 仍可能堵住。

我们因此让等待进程、读取 stdout 和读取 stderr 同时推进。管道句柄先用 child.stdout.take()、child.stderr.take() 取出来：句柄交给读取函数，Child 本身仍留在原处等待和清理。这是一次很具体的所有权转移。

读取函数每次拿到一小块字节，只把剩余配额以内的部分放进缓冲区。超过配额之后继续消费管道，但不继续增加内存；最终把 truncated 设为 true。这里限制的是采集过程，不能等命令全部输出后才缩短字符串。

在 bash_bounds_both_pipes_without_blocking_the_process 中，两个管道各输出十万字节，而各自只保留 4,096 字节。命令仍然能够结束，这同时检验了读取和配额处理。

## 超时之后，进程在哪里

给一个命令设置超时，会让等待有一个期限。期限到了之后，还需要处理已经启动的进程。我们先检查它是否已经退出；如果还活着，就显式 kill 并等待回收。kill_on_drop 作为句柄意外丢弃时的兜底。

实验命令会先把自己的 PID 写入 timeout.pid，然后执行 exec sleep 10。exec 让 sleep 替换当前 Bash，成为我们直接管理的那个进程。测试使用 100ms 的期限，返回后再用 kill -0 检查这个 PID，确认它已经不存在。

还有一种情况值得区分：Bash 自己已经退出，但一个后台后代仍持有输出管道。因此期限必须覆盖进程等待和输出采集两部分。如果我们已观察到父进程退出码 0，却因为管道没有结束而超时，就保留退出码，同时标记 timed_out。这样的结果仍不能当作正常完成。

这个小版本只承诺管理直接子进程。后代进程树的统一清理需要另外的机制，我们把它留作后续专题。

运行 cargo test --locked --test tools bash_。通过这些实验之后，四个工具都能被独立调用。下一章只剩一个关键连接：把执行结果放进模型的下一轮输入，让它根据反馈继续提出动作。

对应代码书签：step-03-bash。实现见 [tools.rs](../../src/tools.rs)，实验记录见[开发记录](../../docs/devlog/03-bash.md)。
