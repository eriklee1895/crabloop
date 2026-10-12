# 两轮真实终端演示

2026-10-12，DeepSeek V4.1 Flash（`deepseek-flash`），原生 Responses API。第一次任务创建并读取 `hello.txt`；第二次只说 “that file”，根据上一轮历史将 Hello 替换成 Bonjour 并读回。共 6 次主请求、5 次工具调用，两轮均返回 Final，未触发摘要。

随后由外部 Python 检查文件内容等于 `Bonjour, CrabLoop!`，并检查第二次用户提问的请求里含有第一次提问和 assistant 回答。[smoke.json](smoke.json) 保存选择后的事实与源码哈希。这个短演示证明本次两轮协作成功，不代表长程成功率。

[demo.cast](demo.cast) 是实际 PTY 字节和时间戳生成的 asciicast v2 录制；[输入脚本](inputs.json) 只在观察到真实提示符后键入任务，没有生成或替换模型输出。[README GIF](../../docs/assets/crabloop-chat.gif) 使用 agg 1.9.0 渲染，96×30 终端、18 px 字体、15 FPS 上限、最长空闲停顿 1.5 秒、末帧 4 秒。原始录制约 12.7 秒，GIF 约 246 KB。已检查开头、工具执行和最终帧。

凭证通过已有配置读取，不在命令输入、录制或公共证据里。临时工作区及完整 JSONL 日志留在本机。PTY 中另以本地 HTTP fixture 验证了 Ctrl-C 取消、失败后阻止继续请求、/new 清除旧历史，结果见 [terminal-check.json](terminal-check.json)；这项离线检查与上面的真实模型演示分开记录。

## 自己录一次

在仓库根目录准备 `.env`，构建后运行下面的命令。每次使用新工作目录和新输出文件。需要 Unix PTY、Python 3 和固定版本 [agg 1.9.0](https://github.com/asciinema/agg/tree/v1.9.0)。

```bash
cargo build --locked
demo_workspace=$(mktemp -d /tmp/crabloop-demo-XXXXXX)
CRABLOOP_PROVIDER=deepseek python3 scripts/record-terminal.py \
  -o .crabloop/my-demo.cast --record-input --cols 96 --rows 30 \
  --steps evidence/interactive/inputs.json \
  -- target/debug/crabloop --workspace "$demo_workspace"
agg --theme github-dark --text-font-family 'JetBrains Mono,Menlo' \
  --font-size 18 --line-height 1.35 --fps-cap 15 \
  --idle-time-limit 1.5 --last-frame-duration 4 \
  .crabloop/my-demo.cast .crabloop/my-demo.gif
cat "$demo_workspace/hello.txt"
```

最后应该看到 `Bonjour, CrabLoop!`。真实模型可能选择不同的工具顺序或回答措辞；检查文件和请求历史，不要求输出逐字复刻本次演示。只在核查录制不包含敏感信息后，选择需要公开的文件。
