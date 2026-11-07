# 收敛后 v1 的真实模型验收（DeepSeek）

时间：2026-10-18 10:54–10:56（北京时间）。模型 `deepseek-flash`，端点 `https://api.deepseek.com/responses`。
`src/` 与提交 `79d0d85` 相同（304 代码行）。原始 stdout/stderr 和生成的工作区只留在本地 `.crabloop/` 与 `playground/`，不入库。

"模型说完成"和"验证通过"分开记录：后者只看独立的验证脚本或我们自己读到的文件。

| 任务 | 模型请求 | 工具调用 | 模型的说法 | 独立验证 |
|---|---|---|---|---|
| 修复任务 `20261018T025400500868Z-coding` | 8 | read 3 · write 2 · bash 4 | 修好了，并贴出了测试输出 | 修复前 verifier 退出码 101，修复后 0；测试文件没动；NOTES.md 已生成 |
| 资料整理 `20261018T025417997789Z-report` | 9 | read 4 · write 2 · bash 6 | 3 项，done 2 / pending 1 | 第一次判失败；原因在验证脚本，不在模型（见下），重查通过 |
| 资料整理 `20261018T025507605988Z-report` | 6 | read 1 · write 2 · bash 4 | 同上 | 用修正后的 verifier 判通过 |
| 多行 stdin 会话 `20261018T025540Z-chat` | 3 + 3 + 3 | 见 JSON | 见下 | 每一行之后都给 note.txt 做了快照 |

**多轮历史。** 第二行只说"把刚才的第二种换掉"，没有给出原文。模型用 `edit_file` 换掉了第 1 行任务写入的那一行，第 1、3 行一个字节都没变。所以同一进程里，后面的任务能看到前面的历史。

**bash 看不到 key。** 每次都是模型自己通过 bash 把计数写进文件，我们再直接读这个文件。
- 父进程只给 `PATH/HOME/LANG` 和 crabloop 的配置时，`env | grep -c OPENAI_API_KEY` 为 0。
- 在本机完整环境里，同样的命令得到 2。这两行来自另外两个环境变量：它们的值里写着 `OPENAI_API_KEY` 这个名字，但它们本身不是这个变量。
- 改用精确匹配 `^OPENAI_API_KEY=` 后，子进程为 0；同一台机器的父 shell 为 1，说明 `env_remove` 确实生效了。
- 另外检查了原始日志和工作区：都不含 DeepSeek key 的值。

**验证脚本的 bug。** 提交 `a9311b4` 要求代码里的文本都用英文，顺手把 `verify-report.py` 期望的负责人从 林/陈/周 改成了 Lin/Chen/Zhou。但 fixture 里写的就是中文，模型照原样抄是对的。
- 现在期望值用 Unicode 转义写回中文。
- 新增测试 `test_expected_owners_match_the_fixture_documents`，把期望值和 fixture 原文绑在一起。
- 失败记录保留，旁边附 `-recheck.json`：没有重新调用模型，只用修正后的脚本重查了一遍。
