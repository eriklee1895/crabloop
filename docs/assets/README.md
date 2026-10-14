# CrabLoop 品牌素材

2026-10-14 选定 T2 方向：现代等宽字形、深海军蓝背景、珊瑚橙蟹身和青绿钳子。吉祥物保持微笑，用独立蟹壳、钳臂与蟹腿形成轮廓。

- [crabloop-banner.png](crabloop-banner.png)：README 首屏，标题 CrabLoop，文案为 “A small agent harness, built step by step in Rust.”。
- [crabloop-mascot.png](crabloop-mascot.png)：透明背景独立吉祥物。

素材使用内置 imagegen 生成，Banner 采用用户选定的 T2 探索稿；独立吉祥物以该 Banner 为输入提取。标题是图像中的字形设计，未绑定具体字体文件，不宣称精确使用某个字体家族。以后若需要可编辑排版，再明确字体与其来源。

两份文件均为 PNG。展示时保持宽高比；保留青绿钳子、珊瑚蟹身和微笑，不添加循环箭头或 API/provider 文案。历史探索稿保留在本地预览目录，不作为额外的公开学习入口。

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
