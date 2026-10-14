# CrabLoop 品牌素材

2026-10-14 选定 T2 方向：现代等宽字形、深海军蓝背景、珊瑚橙蟹身和青绿钳子。吉祥物保持微笑，用独立蟹壳、钳臂与蟹腿形成轮廓。

- [crabloop-banner.png](crabloop-banner.png)：README 首屏，标题 CrabLoop，文案为 “A small agent harness, built step by step in Rust.”。
- [crabloop-mascot.png](crabloop-mascot.png)：透明背景独立吉祥物。

素材使用内置 imagegen 生成，Banner 采用用户选定的 T2 探索稿，并经用户明确授权以本地图像工具裁除上下留白；独立吉祥物以该 Banner 为输入提取。标题是图像中的字形设计，未绑定具体字体文件，不宣称精确使用某个字体家族。以后若需要可编辑排版，再明确字体与其来源。

两份文件均为 PNG。Banner 为 2172 × 543（4:1），从原始 2172 × 724 PNG 顶部裁去 90px、底部裁去 91px，保留区域的像素不变。README 按正文宽度显示，与徽章和正文左边界对齐；窄屏随容器缩小。展示时保持宽高比；保留青绿钳子、珊瑚蟹身和微笑，不添加循环箭头或 API/provider 文案。历史探索稿保留在本地预览目录，不作为额外的公开学习入口。

## 后续编辑入口

最终 PNG 是后续编辑的参考基准，提示词记录设计意图和当时的编辑约束。仅重跑提示词不保证得到相同图像；需要精确取回旧版本时使用 Git 历史。

1. 改 Banner 时，附上仓库中的 `crabloop-banner.png`，说明只修改哪一处；保持 4:1 比例、完整钳尖与蟹腿，以及其余字体、颜色和布局。
2. 改吉祥物时，附上 `crabloop-mascot.png`，保留真实透明背景、微笑、珊瑚橙蟹身和青绿钳子。
3. 每次先生成候选并查看实际尺寸效果；选定后更新正式文件与本说明，通过 Git 保存版本。

可复用的编辑提示词：

```text
Use the attached final CrabLoop asset as the edit target.
Change only: [describe the requested change].
Preserve all other elements, especially the smiling pixel crab, coral-orange shell,
teal claws, and modern geek typography. Do not add loop arrows or provider/API text.
For the banner, retain the 4:1 canvas and keep the full title, subtitle and mascot visible.
For the standalone mascot, retain genuine alpha transparency.
```

## 来源与精确恢复

生成方式为内置 imagegen；本次没有记录可固定重放的模型版本或随机种子。图中的标题没有独立字体文件。

- [最初归档的 T2 原图与透明吉祥物](https://github.com/eriklee1895/crabloop/tree/f35404d/docs/assets)：Banner 为 2172 × 724。
- [4:1 裁切版本](https://github.com/eriklee1895/crabloop/tree/30a9ed7/docs/assets)：经用户明确授权，使用 Pillow 的 `crop((0, 90, 2172, 633))` 得到 2172 × 543，保留区域像素不变。
- 历史探索稿不作为必需依赖；下面的 S1 是当时的试稿名。今后直接使用本目录的最终图片作为编辑参考。

## 最终 Banner 提示词

输入为已选定的 S1 微笑像素蟹 Banner；以下为 T2 字形编辑提示词。

```text
Use case: text-localization / typography refinement. Asset: edit the supplied CrabLoop banner, concept T2.
Change ONLY the typography in the left-hand text area. Preserve the smiling coral-orange crab and turquoise-teal claws on the right exactly: same shape, pixels, face, pose, size and position. Preserve the dark midnight-navy background and overall canvas ratio. Preserve the original text content and colors: warm off-white title and subdued teal subtitle.
Typography direction: Use a SINGLE coordinated contemporary programming monospace family for both main title and subtitle. Main title is bold, slightly condensed, precise but not stencil-like, with modern squared curves and smooth edges; no pixel stairs or arcade lettering. Subtitle uses a regular-to-medium weight of the SAME FAMILY, clearly smaller, relaxed spacing. Contemporary terminal-tool identity with technical confidence, not vintage computing nostalgia.
Exact main title: "CrabLoop"
Exact subtitle on two lines: "A small agent harness," and "built step by step in Rust."
Keep all text left-aligned, clear at GitHub README width, with balanced title/subtitle spacing and clean margins. Title should feel like a modern serious geek tool with an approachable mascot. DO NOT turn the letters into pixel art. Do not add extra labels, decorative lines, icons, arrows, circles, provider names, watermarks or lighting effects. Do not alter the crab.
```

## 独立吉祥物提示词

输入为已选定的最终 Banner。

```text
Use case: background-extraction.
Asset: standalone CrabLoop mascot PNG with genuine transparent background, derived from the selected banner.
Input image is the edit target. Extract ONLY the smiling pixel crab on the right. Preserve its exact pose, silhouette, square pixel shapes, coral-orange shell, turquoise-teal open pincers, cream highlights, six short legs, black square eyes and small upward smile. Remove ALL text and the navy background. Do not redesign, smooth, embellish, recolor or add anything. Place the entire extracted crab centered on a square transparent canvas with modest even padding, all claws and feet visible. No background, no floor, no shadow, no fake checkerboard. This is the same character as the banner, used as a reusable project mascot.
```
