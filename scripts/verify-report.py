#!/usr/bin/env python3
"""Independent fact and source-preservation checks for the local report task."""
import json
from pathlib import Path
import sys
root = Path(__file__).resolve().parents[1]
workspace = Path(sys.argv[1])
try:
    data = json.loads((workspace / "report.json").read_text())
    expected = [{"id": ident, "owner": owner, "status": status, "source": f"inbox/{ident}.md"}
                for ident, owner, status in [("alpha", "林", "done"), ("beta", "陈", "pending"), ("gamma", "周", "done")]]
    actual = [{key: item[key] for key in ("id", "owner", "status", "source")} for item in data["items"]]
    assert actual == expected, "item facts or sources do not match"
    counts = data["counts"]
    assert {key: counts[key] for key in ("done", "pending")} == {"done": 2, "pending": 1}, "counts do not match"
    assert all(type(counts[key]) is int for key in ("done", "pending")), "counts must be integers"
    if "total" in counts:
        assert type(counts["total"]) is int and counts["total"] == 3, "optional total is incorrect"
    summary = (workspace / "SUMMARY.md").read_text()
    for item in expected:
        source = item["source"]
        assert source in summary, "summary must cite each source"
        assert (workspace / source).read_bytes() == (root / "fixtures/local-report" / source).read_bytes(), "source document changed"
except (OSError, ValueError, AssertionError, KeyError, TypeError) as error:
    print("report verification failed:", error, file=sys.stderr)
    raise SystemExit(1)
print("report verification passed: 3 items, exact facts and counts, all sources preserved")
