# 03：Bash、管道和超时

日期：2026-10-11。代码书签：step-03-bash。

四项 Bash 测试先因没有执行器而失败：非零退出与 stderr、工作目录/关闭 stdin、大量双管道输出、超时后回收直接子进程。实现 Bash 分支、有限 drain 和整体 timeout 后通过。

再增加“父进程退出而后代持有管道”实验，发现超时分支丢失了已经观察到的退出码。新增测试先得到 null 而失败，修正为保留 try_wait 的结果后再回归。timed_out 和 exit_code 分别表示等待期限与直接子进程的状态。

源码只承诺回收直接子进程；测试中的短暂后台 sleep 自行退出。没有把这项验证描述为完整进程树隔离。

验证命令：cargo test --locked；cargo clippy --locked --all-targets -- -D warnings。实际 Bash 和文件执行均发生在临时目录。
