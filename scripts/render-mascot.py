#!/usr/bin/env python3
"""Build the small transparent mascot GIF from the archived artwork and timeline."""
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "docs/assets"
SOURCE = ASSETS / "mascot-animation"
FRAMES = SOURCE / "frames"


def build_poses(size, motions):
    original = Image.open(ASSETS / "crabloop-mascot.png").convert("RGBA")
    closed = Image.open(SOURCE / "blink-source.png").convert("RGBA")
    if original.size != (1254, 1254) or closed.size != original.size:
        raise ValueError("Revisit the eye regions and claw pivot when replacing the source artwork")

    # Use only the generated eye patches to preserve the original character.
    eye_mask = Image.new("L", original.size)
    draw = ImageDraw.Draw(eye_mask)
    for box in [(408, 636, 524, 764), (728, 636, 843, 764)]:
        draw.rectangle(box, fill=255)
    eye_mask = eye_mask.filter(ImageFilter.GaussianBlur(2))
    blink = Image.composite(closed, original, eye_mask)
    half = blink.copy()
    for left, top, right, bottom in [(414, 642, 518, 757), (734, 642, 835, 757)]:
        eye = original.crop((left, top, right, bottom))
        height = (bottom - top) // 2
        eye = eye.resize((right - left, height), Image.Resampling.NEAREST)
        half.paste(eye, (left, top + (bottom - top - height) // 2))
    poses = {"idle": original, "half-blink": half, "blink": blink}

    # Isolate the viewer-right teal claw plus its upper arm, keeping an overlap at the joint.
    claw_mask = Image.new("L", original.size)
    pixels, mask = original.load(), claw_mask.load()
    for y in range(240, 650):
        for x in range(840, 1180):
            red, green, blue, alpha = pixels[x, y]
            teal = green - red > 10 and blue - red > 10 and y < 590
            arm = 978 <= x < 1080 and 575 <= y < 650
            if alpha and (teal or arm):
                mask[x, y] = 255
    claw = original.copy()
    claw.putalpha(ImageChops.multiply(original.getchannel("A"), claw_mask))
    removal = claw_mask.copy()
    ImageDraw.Draw(removal).rectangle((0, 620, 1254, 1254), fill=0)
    body = original.copy()
    body.putalpha(ImageChops.multiply(original.getchannel("A"), ImageChops.invert(removal)))
    for name, motion in motions.items():
        moved = claw.rotate(motion["angle"], resample=Image.Resampling.NEAREST, center=(1028, 640))
        poses[name] = Image.alpha_composite(body, moved) if motion["angle"] else original.copy()

    frames = {}
    for name, pose in poses.items():
        frame = pose.resize((size, size), Image.Resampling.NEAREST)
        dy = round(motions.get(name, {}).get("dy", 0) * size / 256)
        if dy:
            shifted = Image.new("RGBA", frame.size)
            shifted.alpha_composite(frame, (0, dy))
            frame = shifted
        frames[name] = frame
    return frames


def export_gif(poses, timeline):
    # Share one palette to prevent color flicker; reserve index 255 for binary GIF transparency.
    size = next(iter(poses.values())).size
    atlas = Image.new("RGB", (size[0] * len(poses), size[1]))
    for index, frame in enumerate(poses.values()):
        atlas.paste(frame.convert("RGB"), (size[0] * index, 0))
    colors = atlas.quantize(colors=255, method=Image.Quantize.MEDIANCUT).getpalette()[:765]
    palette = Image.new("P", (1, 1))
    palette.putpalette(colors + [255, 0, 255])
    indexed = {}
    for name, frame in poses.items():
        result = frame.convert("RGB").quantize(palette=palette, dither=Image.Dither.NONE)
        transparent = frame.getchannel("A").point(lambda value: 255 if value < 128 else 0)
        result.paste(255, mask=transparent)
        indexed[name] = result

    frames = [indexed[item["pose"]] for item in timeline["frames"]]
    durations = [item["duration_ms"] for item in timeline["frames"]]
    output = ASSETS / "crabloop-mascot.gif"
    frames[0].save(output, save_all=True, append_images=frames[1:], duration=durations,
                   loop=timeline["loop"], transparency=255, background=255,
                   disposal=2, optimize=False)
    return output, sum(durations)


def main():
    timeline = json.loads((SOURCE / "timeline.json").read_text())
    poses = build_poses(timeline["size"], timeline["motions"])
    FRAMES.mkdir(exist_ok=True)
    for name, frame in poses.items():
        frame.save(FRAMES / f"{name}.png", optimize=True)
    output, duration = export_gif(poses, timeline)
    print(f"{output.relative_to(ROOT)}: {timeline['size']}px, {duration}ms, {output.stat().st_size} bytes")


if __name__ == "__main__":
    main()
