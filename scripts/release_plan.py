#!/usr/bin/env python3
"""Decide what the stable release workflow should do for the current commit.

Modes:
  bump     Cargo.toml version == latest stable tag and there are new commits
  release  Cargo.toml version is newer than the latest stable tag
  skip     nothing to do (no new commits, or release already exists)
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

SEMVER_TAG_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
CARGO_VERSION_RE = re.compile(r'^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', re.MULTILINE)


def run(*args: str) -> str:
    return subprocess.check_output(args, text=True).strip()


def read_cargo_version(path: str) -> tuple[tuple[int, int, int], str]:
    match = CARGO_VERSION_RE.search(Path(path).read_text())
    if not match:
        sys.exit(f"Could not read version from {path}")
    parts = tuple(int(part) for part in match.groups())
    return parts, ".".join(match.groups())


def latest_stable_tag() -> tuple[tuple[int, int, int], str | None]:
    output = run("git", "tag", "--list", "v[0-9]*", "--sort=-v:refname")
    for line in output.splitlines():
        line = line.strip()
        match = SEMVER_TAG_RE.match(line)
        if match:
            return tuple(int(part) for part in match.groups()), line
    return (0, 0, 0), None


def release_exists(version: str) -> bool:
    result = subprocess.run(
        ["gh", "release", "view", f"v{version}"],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cargo", default="Cargo.toml")
    parser.add_argument(
        "--output",
        default=os.environ.get("GITHUB_OUTPUT"),
        help="File to append GitHub Actions outputs to",
    )
    args = parser.parse_args()

    current, current_str = read_cargo_version(args.cargo)
    latest, tag = latest_stable_tag()

    commits_since_tag = 0
    if tag:
        commits_since_tag = int(run("git", "rev-list", "--count", f"{tag}..HEAD") or 0)
    else:
        commits_since_tag = int(run("git", "rev-list", "--count", "HEAD") or 0)

    if current > latest:
        mode = "release"
        version = current_str
        if release_exists(version):
            mode = "skip"
    elif current == latest and commits_since_tag > 0:
        mode = "bump"
        version = f"{current[0]}.{current[1]}.{current[2] + 1}"
    else:
        mode = "skip"
        version = current_str

    values = {
        "mode": mode,
        "version": version,
        "current": current_str,
        "latest": tag or "",
        "commits": str(commits_since_tag),
    }

    for key, value in values.items():
        print(f"{key}={value}")

    if args.output:
        with open(args.output, "a", encoding="utf-8") as handle:
            for key, value in values.items():
                handle.write(f"{key}={value}\n")


if __name__ == "__main__":
    main()
