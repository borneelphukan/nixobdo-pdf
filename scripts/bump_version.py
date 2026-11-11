#!/usr/bin/env python3
"""Bump the project version and add a CHANGELOG section.

Updates Cargo.toml, Cargo.lock, and CHANGELOG.md so a release build can use
`cargo build --locked`. The Windows installer takes its version from the
release workflow defines and keeps a neutral `0.0.0` fallback, so it is not
rewritten here.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections import OrderedDict
from datetime import datetime, timezone
from pathlib import Path

VERSION = r"\d+\.\d+\.\d+"
SECTION_ORDER = [
    "Added",
    "Changed",
    "Fixed",
    "Performance",
    "Documentation",
    "Internal",
    "Other",
]
KIND_TO_SECTION = {
    "feat": "Added",
    "fix": "Fixed",
    "perf": "Performance",
    "refactor": "Changed",
    "docs": "Documentation",
    "build": "Internal",
    "chore": "Internal",
    "ci": "Internal",
    "style": "Internal",
    "test": "Internal",
}


def read(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    Path(path).write_text(content, encoding="utf-8")


def replace_once(path: str, pattern: str, replacement: str, label: str) -> None:
    content = read(path)
    updated, count = re.subn(pattern, replacement, content, count=1, flags=re.MULTILINE)
    if count != 1:
        sys.exit(f"Could not update {label} in {path}")
    write(path, updated)


def update_cargo(version: str) -> None:
    replace_once(
        "Cargo.toml",
        rf'^(version\s*=\s*"){VERSION}(")',
        rf"\g<1>{version}\g<2>",
        "package version",
    )


def update_lock(version: str) -> None:
    content = read("Cargo.lock")
    pattern = re.compile(
        r'(\[\[package\]\]\nname = "nixobdo-pdf"\nversion = ")[^"]+(")'
    )
    updated, count = pattern.subn(rf"\g<1>{version}\g<2>", content, count=1)
    if count != 1:
        sys.exit("Could not update nixobdo-pdf version in Cargo.lock")
    write("Cargo.lock", updated)


def git_subjects(since_tag: str) -> list[str]:
    revision = f"{since_tag}..HEAD" if since_tag else "HEAD"
    try:
        output = subprocess.check_output(
            ["git", "log", "--no-merges", "--format=%s", revision],
            text=True,
        )
    except subprocess.CalledProcessError as error:
        sys.exit(f"git log failed for {since_tag}..HEAD: {error}")
    return [line.strip() for line in output.splitlines() if line.strip()]


def classify(subject: str) -> tuple[str, str]:
    subject = re.sub(r"\s*\(#\d+\)$", "", subject).strip()
    match = re.match(r"^([A-Za-z]+)(?:\([^)]*\))?!?:\s*(.+)$", subject)
    if match:
        kind = match.group(1).lower()
        message = match.group(2).strip()
    else:
        kind = ""
        message = subject
    return KIND_TO_SECTION.get(kind, "Other"), message


def build_section(version: str, since_tag: str) -> str:
    grouped: OrderedDict[str, list[str]] = OrderedDict()

    for subject in git_subjects(since_tag):
        if subject.lower().startswith("chore: bump version"):
            continue
        section, message = classify(subject)
        grouped.setdefault(section, []).append(message)

    date = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    lines = [f"## [{version}] - {date}"]

    if not grouped:
        lines.extend(["### Other", "- No user-facing changes."])
        return "\n".join(lines) + "\n"

    for section in SECTION_ORDER:
        messages = grouped.get(section)
        if not messages:
            continue
        lines.append(f"### {section}")
        lines.extend(f"- {message}" for message in messages)

    return "\n".join(lines) + "\n"


def upsert_changelog(version: str, since_tag: str) -> None:
    path = "CHANGELOG.md"
    section = build_section(version, since_tag)

    try:
        content = read(path)
    except FileNotFoundError:
        write(path, "# Changelog\n\n" + section)
        return

    heading = re.compile(
        rf"(?ms)^## \[{re.escape(version)}\][^\n]*\n.*?(?=^## \[|\Z)"
    )
    if heading.search(content):
        content = heading.sub(section + "\n", content, count=1)
    else:
        first_section = re.search(r"(?m)^## \[", content)
        if first_section:
            index = first_section.start()
            content = content[:index] + section + "\n" + content[index:]
        else:
            content = content.rstrip() + "\n\n" + section

    write(path, content)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True, help="New version, e.g. 0.1.38")
    parser.add_argument(
        "--from-tag",
        default="",
        help="Previous release tag used to collect changelog entries, e.g. v0.1.37",
    )
    args = parser.parse_args()

    if not re.fullmatch(VERSION, args.version):
        sys.exit(f"Invalid version: {args.version}")

    update_cargo(args.version)
    update_lock(args.version)
    upsert_changelog(args.version, args.from_tag)

    print(f"Bumped nixobdo-pdf to {args.version}")


if __name__ == "__main__":
    main()
