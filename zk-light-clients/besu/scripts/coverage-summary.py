"""Render LLVM coverage totals as a compact Markdown table."""

import argparse
import json
from pathlib import Path


def render(report):
    totals = [entry["totals"] for entry in report["data"]]
    if not totals:
        raise ValueError("coverage report contains no totals")
    rows = []
    for label, key in (
        ("Lines", "lines"),
        ("Statements", None),
        ("Branches", "branches"),
        ("Functions", "functions"),
        ("Regions", "regions"),
    ):
        if key is None or any(key not in total for total in totals):
            rows.append((label, "N/A", "N/A"))
            continue
        count = sum(total[key]["count"] for total in totals)
        covered = sum(total[key]["covered"] for total in totals)
        percent = f"{100 * covered / count:.2f}%" if count else "N/A"
        rows.append((label, f"{covered} / {count}", percent))
    lines = ["| Measure | Covered | % |", "| --- | --- | --- |"]
    lines.extend(f"| {label} | {covered} | {percent} |" for label, covered, percent in rows)
    lines.extend([
        "",
        "Statements: LLVM reports regions, not a separate statement metric.",
        "Branches: N/A means unavailable or no instrumented branches; it does not mean 100%.",
    ])
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path, help="cargo llvm-cov report --json output")
    args = parser.parse_args()
    try:
        print(render(json.loads(args.report.read_text())))
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"Invalid coverage report: {error}\n")


if __name__ == "__main__":
    main()
