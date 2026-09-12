#!/usr/bin/env python3
"""Regenerate src/earth/ut1_table.rs from IERS EOP C04 14.

Downloads the legacy IAU2000A224 one-file series (0h UTC, UT1-UTC column)
and emits 5-day DUT1 knots from 1972-01-01 through the last published row.

Usage:
    python3 scripts/gen_ut1_table.py [--input PATH]

Default input URL:
    https://datacenter.iers.org/data/latestVersion/224_EOP_C04_14.62-NOW.IAU2000A224.txt
"""

from __future__ import annotations

import argparse
import sys
import urllib.request
from pathlib import Path

DEFAULT_URL = (
    "https://datacenter.iers.org/data/latestVersion/"
    "224_EOP_C04_14.62-NOW.IAU2000A224.txt"
)
OUT = Path(__file__).resolve().parent.parent / "src" / "earth" / "ut1_table.rs"
FIRST_MJD = 41317  # 1972-01-01 0h UTC
KNOT_SPACING = 5


def parse_c04(text: str) -> dict[int, float]:
    data: dict[int, float] = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "FORMAT" in line:
            continue
        if any(
            token in line
            for token in (
                "EARTH",
                "INTERNATIONAL",
                "Description",
                "contact",
                "Date",
                '"',
                "(0h",
                "###",
            )
        ):
            continue
        parts = line.split()
        if len(parts) < 7:
            continue
        try:
            mjd = int(parts[3])
            dut1 = float(parts[6])
        except (ValueError, IndexError):
            continue
        data[mjd] = dut1
    return data


def emit_table(data: dict[int, float]) -> str:
    end_mjd = max(data.keys())
    knots: list[tuple[int, int]] = []
    mjd = FIRST_MJD
    while mjd <= end_mjd:
        if mjd not in data:
            raise SystemExit(f"missing C04 row for MJD {mjd}")
        tenths_ms = round(data[mjd] * 10_000)
        if not -32768 <= tenths_ms <= 32767:
            raise SystemExit(f"DUT1 {data[mjd]} s out of i16 tenths-ms range at MJD {mjd}")
        knots.append((mjd, tenths_ms))
        mjd += KNOT_SPACING

    lines = [
        "//! Auto-generated from IERS EOP C04 14 (IAU2000A224).",
        "//! Do not edit by hand; regenerate with `scripts/gen_ut1_table.py`.",
        "",
        "/// Last MJD in the pin (0h UTC).",
        f"pub const LAST_MJD: i32 = {end_mjd};",
        "",
        "/// First MJD in the pin (1972-01-01 0h UTC).",
        f"pub const FIRST_MJD: i32 = {FIRST_MJD};",
        "",
        "/// Knot spacing in days.",
        f"pub const KNOT_SPACING: i32 = {KNOT_SPACING};",
        "",
        "/// `(mjd, dut1_tenths_ms)` knots: DUT1 in units of 0.1 ms.",
        "pub const KNOTS: &[(i32, i16)] = &[",
    ]
    for i, (m, t) in enumerate(knots):
        comma = "," if i < len(knots) - 1 else ""
        lines.append(f"    ({m}, {t}){comma}")
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input",
        help="Local C04 file path (default: download from IERS)",
    )
    parser.add_argument(
        "--url",
        default=DEFAULT_URL,
        help="C04 download URL when --input is omitted",
    )
    args = parser.parse_args()

    if args.input:
        text = Path(args.input).read_text()
    else:
        with urllib.request.urlopen(args.url, timeout=120) as resp:
            text = resp.read().decode()

    data = parse_c04(text)
    if FIRST_MJD not in data:
        raise SystemExit(f"C04 file does not cover MJD {FIRST_MJD}")

    OUT.write_text(emit_table(data))
    print(f"wrote {OUT} ({len(data)} daily rows, last MJD {max(data)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
