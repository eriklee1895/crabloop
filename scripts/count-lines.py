#!/usr/bin/env python3
"""Count every Rust source under src; comments/blank lines have an explicit rule."""
from pathlib import Path
import json
import argparse
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--max", type=int, help="fail when code lines exceed this number")
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
files = []
for path in sorted((root / "src").rglob("*.rs")):
    lines = path.read_text().splitlines()
    code = sum(bool(line.strip()) and not line.lstrip().startswith("//") for line in lines)
    files.append({"path": str(path.relative_to(root)), "physical": len(lines), "code": code})
print(json.dumps({"rule": "all src/**/*.rs; nonblank and non-standalone // comment lines; block comments count", "files": files, "physical": sum(f["physical"] for f in files), "code": sum(f["code"] for f in files)}, indent=2))

if args.max is not None and sum(f["code"] for f in files) > args.max:
    raise SystemExit(1)
