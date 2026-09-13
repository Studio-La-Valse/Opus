#!/usr/bin/env python3
"""Writes samples.json: one {id, file, title, bytes} entry per .musicxml file
in the given directory, sorted by filename. `title` is read from
<work-title>, falling back to <movement-title>, falling back to the filename
- so dropping a new sample into assets/xmlsamples/ needs no edit here.

Usage: generate-samples-json.py <samples-dir> <out-file>
"""

import json
import re
import sys
from pathlib import Path

TITLE_TAGS = ("work-title", "movement-title")


def sample_title(path: Path) -> str:
    text = path.read_text(encoding="utf-8", errors="replace")
    for tag in TITLE_TAGS:
        match = re.search(rf"<{tag}>(.*?)</{tag}>", text, re.DOTALL)
        if match:
            title = match.group(1).strip()
            if title:
                return title
    return path.stem


def main() -> None:
    src_dir = Path(sys.argv[1])
    out_path = Path(sys.argv[2])

    samples = [
        {
            "id": path.stem,
            "file": path.name,
            "title": sample_title(path),
            "bytes": path.stat().st_size,
        }
        for path in sorted(src_dir.glob("*.musicxml"))
    ]

    out_path.write_text(json.dumps(samples, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
