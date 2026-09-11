#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Enforce Go/Rust cyclomatic complexity without building any project code."""

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parent.parent
LIMITS = Path(__file__).with_name("complexity-limits.json")
EXTENSIONS = {"go": ".go", "rust": ".rs"}
GENERATED = re.compile(
    r"^\s*//.*(?:do\s+not\s+edit|code\s+generated|generated\s+by|@generated|auto-generated\s+file)",
    re.IGNORECASE | re.MULTILINE,
)
EXCLUDED_DIRS = {"node_modules", "target", "out", "vendor", "generated"}


def source_files(root, language):
    """Include new local files as well as tracked files, using Git's ignores."""
    result = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard",
         "--", "*" + EXTENSIONS[language]],
        cwd=root, check=True, capture_output=True,
    )
    files = []
    for name in sorted(set(result.stdout.decode().split("\0")) - {""}):
        path = root / name
        if path.is_symlink() or not path.is_file():
            continue
        if EXCLUDED_DIRS.intersection(Path(name).parts):
            continue
        if GENERATED.search("\n".join(path.read_text().splitlines()[:10])):
            continue
        files.append(name)
    return files


def go_metrics(root, files):
    output = subprocess.run(
        ["gocyclo", *files], cwd=root, check=True, capture_output=True, text=True,
    ).stdout
    for line in output.splitlines():
        complexity, _package, function, position = line.split(" ", 3)
        path, line_number, _column = position.rsplit(":", 2)
        yield path, function, int(line_number), int(complexity)


def rust_functions(space, path, parents=()):
    """RCA sums nested spaces; subtract them to measure each function itself."""
    name = space["name"] or "<anonymous>"
    names = parents if space["kind"] == "unit" else (*parents, name)
    if space["kind"] in {"function", "closure"}:
        complexity = space["metrics"]["cyclomatic"]["sum"] - sum(
            child["metrics"]["cyclomatic"]["sum"] for child in space["spaces"]
        )
        yield path, "::".join(names), space["start_line"], int(complexity)
    for child in space["spaces"]:
        yield from rust_functions(child, path, names)


def rust_metrics(root, files):
    # Explicit paths also work in hidden Herdr worktrees, which RCA's directory
    # walker can silently skip. Bound batches to stay below OS argument limits.
    decoder = json.JSONDecoder()
    for start in range(0, len(files), 100):
        batch = files[start:start + 100]
        output = subprocess.run(
            ["rust-code-analysis-cli", "-m", "-O", "json",
             *[arg for name in batch for arg in ("-p", name)]],
            cwd=root, check=True, capture_output=True, text=True,
        ).stdout
        seen = set()
        while output.strip():
            output = output.lstrip()
            space, end = decoder.raw_decode(output)
            output = output[end:]
            seen.add(space["name"])
            yield from rust_functions(space, space["name"])
        if seen != set(batch):
            raise ValueError(f"Rust analyzer did not report all input files: {set(batch) - seen}")


def violations(metrics, config):
    exceptions = config.get("exceptions", {})
    seen = set()
    for path, function, line, complexity in metrics:
        key = f"{path}::{function}"
        seen.add(key)
        limit = exceptions.get(key, {}).get("max", config["max"])
        if complexity > limit:
            yield f"{path}:{line}: {function} has cyclomatic complexity {complexity} (max {limit})"
        elif key in exceptions and complexity < limit:
            yield f"{path}:{line}: lower or remove the stale complexity exception for {function} ({complexity} < {limit})"
    for key in sorted(exceptions.keys() - seen):
        yield f"Remove the stale complexity exception for {key}"


def check(root, language, config):
    files = source_files(root, language)
    if not files:
        raise ValueError(f"No {language} source files found")
    analyze = go_metrics if language == "go" else rust_metrics
    # A batch also bounds gocyclo's argument list for large repositories.
    metrics = [metric for start in range(0, len(files), 100)
               for metric in analyze(root, files[start:start + 100])]
    if not metrics:
        raise ValueError(f"No {language} functions analyzed")
    errors = list(violations(metrics, config))
    for error in errors:
        print(error, file=sys.stderr)
    print(f"{language}: checked {len(metrics)} functions in {len(files)} files "
          f"(max {config['max']}, {len(errors)} violations)")
    return bool(errors)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("language", choices=EXTENSIONS)
    args = parser.parse_args()
    try:
        config = json.loads(LIMITS.read_text())[args.language]
        return check(ROOT, args.language, config)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"Complexity check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
