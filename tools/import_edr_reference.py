#!/usr/bin/env python3
"""Convert the controlled BouwZo ISSO 54 reference workbook to JSON fixtures.

The source workbook is intentionally kept outside the public repository. This
script records its SHA-256 and extracts only numerical reference data into a
machine-readable fixture directory.

Usage:
  python tools/import_edr_reference.py /path/to/Referentiewaarden.xlsx data/edr/reference

The importer fails closed on duplicate test IDs, duplicate output posts, or a
workbook structure that does not match the expected ISSO 54 sheets.
"""

from __future__ import annotations

import hashlib
import json
import math
import pathlib
import sys
from typing import Any

try:
    import openpyxl
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Install development dependency: pip install -r requirements-dev.txt") from exc

SHEETS = {
    "Basistests": "base-reference.json",
    "Realistische gebouwen": "realistic-reference.json",
    "Maatwerkadvies": "mwa-reference.json",
}


def finite_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and math.isfinite(value)


def parse_sheet(ws):
    tests: list[tuple[int, str]] = []
    for col in range(5, ws.max_column + 1):
        value = ws.cell(4, col).value
        if value is not None:
            test_id = str(value).strip()
            if test_id:
                tests.append((col, test_id))

    ids = [test_id for _, test_id in tests]
    if len(ids) != len(set(ids)):
        raise ValueError(f"Duplicate test IDs in {ws.title}")

    outputs = []
    posts: set[str] = set()
    for row in range(6, ws.max_row + 1):
        post = ws.cell(row, 3).value
        if post is None:
            continue
        post = str(post).strip()
        if not post:
            continue
        if post in posts:
            raise ValueError(f"Duplicate output post {post!r} in {ws.title}")
        posts.add(post)

        values = {}
        for col, test_id in tests:
            value = ws.cell(row, col).value
            if finite_number(value):
                values[test_id] = float(value)
            elif value is not None:
                values[test_id] = value

        outputs.append({
            "post": post,
            "unit": None if ws.cell(row, 4).value is None else str(ws.cell(row, 4).value).strip(),
            "values": values,
        })

    return {"tests": ids, "outputs": outputs}


def main() -> int:
    if len(sys.argv) != 3:
        raise SystemExit("Usage: import_edr_reference.py SOURCE.xlsx OUTPUT_DIR")

    source = pathlib.Path(sys.argv[1]).resolve()
    output_dir = pathlib.Path(sys.argv[2]).resolve()
    if not source.is_file():
        raise SystemExit(f"Source workbook not found: {source}")

    output_dir.mkdir(parents=True, exist_ok=True)
    sha256 = hashlib.sha256(source.read_bytes()).hexdigest()
    workbook = openpyxl.load_workbook(source, data_only=True, read_only=True)

    missing = [name for name in SHEETS if name not in workbook.sheetnames]
    if missing:
        raise SystemExit(f"Missing expected ISSO 54 sheets: {', '.join(missing)}")

    index = {
        "source_file": source.name,
        "sha256": sha256,
        "sheets": {},
        "total_tests": 0,
        "total_populated_values": 0,
    }

    for sheet, filename in SHEETS.items():
        parsed = parse_sheet(workbook[sheet])
        populated = sum(len(row["values"]) for row in parsed["outputs"])
        index["sheets"][sheet] = {
            "tests": len(parsed["tests"]),
            "output_rows": len(parsed["outputs"]),
            "populated_values": populated,
        }
        index["total_tests"] += len(parsed["tests"])
        index["total_populated_values"] += populated
        (output_dir / filename).write_text(
            json.dumps(parsed, ensure_ascii=False, separators=(",", ":")) + "\n",
            encoding="utf-8",
        )

    (output_dir / "reference-index.json").write_text(
        json.dumps(index, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(json.dumps(index, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
