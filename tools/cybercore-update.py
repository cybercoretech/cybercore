#!/usr/bin/env python3
"""Safely propagate a published Cybercore release to local consumers."""

from __future__ import annotations

import argparse
import curses
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


SCRIPT = Path(__file__).resolve()
FRAMEWORK = SCRIPT.parents[1]
MANIFEST = SCRIPT.with_name("cybercore-consumers.json")


def command(
    args: list[str],
    cwd: Path | None = None,
    check: bool = True,
    show_output: bool = True,
) -> subprocess.CompletedProcess[str]:
    print("$", " ".join(args))
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True)
    if show_output and result.stdout:
        print(result.stdout.rstrip())
    if show_output and result.stderr:
        print(result.stderr.rstrip(), file=sys.stderr)
    if check and result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(args)}")
    return result


def source_path(raw: str) -> Path:
    path = Path(os.path.expanduser(raw))
    return path if path.is_absolute() else (FRAMEWORK / path).resolve()


def has_only_allowed_changes(path: Path, allowed: list[str]) -> bool:
    result = subprocess.run(
        ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"],
        text=True,
        capture_output=True,
        check=True,
    )
    if not result.stdout.strip():
        return True
    allowed_paths = {Path(item) for item in allowed}
    for line in result.stdout.splitlines():
        relative = Path(line[3:])
        if not any(relative == allowed_path or allowed_path in relative.parents for allowed_path in allowed_paths):
            return False
    return True


def metadata(source: Path, package: str) -> dict[str, Any]:
    result = command(
        [
            "cargo",
            "metadata",
            "--manifest-path",
            str(source / "Cargo.toml"),
            "--format-version",
            "1",
            "--no-deps",
        ],
        check=True,
        show_output=False,
    )
    data = json.loads(result.stdout)
    for item in data["packages"]:
        if item["name"] == package:
            return {"target_directory": Path(data["target_directory"]), "package": item}
    raise RuntimeError(f"package {package} not found in {source}")


def install_binary(binary: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix=f".{destination.name}.", dir=destination.parent, delete=False) as handle:
        temporary = Path(handle.name)
    try:
        shutil.copy2(binary, temporary)
        temporary.chmod(0o755)
        os.replace(temporary, destination)
    finally:
        temporary.unlink(missing_ok=True)


def process_consumer(item: dict[str, Any], apply: bool) -> str:
    source = source_path(item["source"])
    if not source.is_dir():
        print(f"BLOCKED {item['name']}: source does not exist: {source}")
        return "blocked"
    if not has_only_allowed_changes(source, item.get("allowed_dirty", [])):
        print(f"BLOCKED {item['name']}: worktree is dirty: {source}")
        return "blocked"

    info = metadata(source, item["package"])
    manifest = info["package"]["manifest_path"]
    dependency_text = Path(manifest).read_text(encoding="utf-8")
    if re.search(r"(?m)^\s*cybercore\s*=.*version\s*=\s*[\"']0\.4", dependency_text):
        print(f"BLOCKED {item['name']}: declares incompatible cybercore 0.4 dependency")
        return "blocked"

    if not apply:
        print(f"READY {item['name']}: would update dependencies, build, and install")
        return "ready"

    if "cybercore" in dependency_text:
        command(["cargo", "update", "-p", "cybercore"], cwd=source)
    build_args = ["cargo", "build", "--release", "--locked", "--package", item["package"]]
    if item.get("features"):
        build_args.extend(["--features", ",".join(item["features"])])
    command(build_args, cwd=source)
    binary = info["target_directory"] / "release" / item["binary"]
    if not binary.is_file():
        raise RuntimeError(f"built binary not found: {binary}")
    destination = Path(os.path.expanduser(item["install"]))
    install_binary(binary, destination)
    if item.get("service"):
        command(["systemctl", "--user", "restart", item["service"]])
    print(f"UPDATED {item['name']}: {destination}")
    return "updated"


def process_sync(item: dict[str, Any], apply: bool) -> str:
    source = source_path(item["source"])
    if not source.is_dir():
        print(f"BLOCKED {item['name']}: source does not exist: {source}")
        return "blocked"
    if not has_only_allowed_changes(source, item.get("allowed_dirty", [])):
        print(f"BLOCKED {item['name']}: worktree is dirty: {source}")
        return "blocked"
    if not apply:
        print(f"READY {item['name']}: would regenerate canonical theme snapshot")
        return "ready"
    command(item["command"], cwd=source)
    print(f"UPDATED {item['name']}")
    return "updated"


def run_tui(config: dict[str, Any]) -> list[dict[str, Any]] | None:
    entries = [
        {**item, "kind": "consumer"} for item in config["consumers"]
    ] + [
        {**item, "kind": "sync"} for item in config["syncs"]
    ]

    def screen(stdscr: Any) -> list[dict[str, Any]] | None:
        curses.curs_set(0)
        selected = [True] * len(entries)
        cursor = 0
        while True:
            stdscr.erase()
            height, width = stdscr.getmaxyx()
            title = "CYBERCORE // SYSTEM UPDATER"
            stdscr.addnstr(0, 0, title, max(0, width - 1), curses.A_BOLD)
            stdscr.addnstr(
                1,
                0,
                "Space toggle  A all  N none  Enter apply  Q cancel",
                max(0, width - 1),
            )
            for row, item in enumerate(entries[: max(0, height - 4)]):
                marker = "[x]" if selected[row] else "[ ]"
                prefix = ">" if row == cursor else " "
                label = f"{prefix} {marker} {item['name']}"
                if item["kind"] == "sync":
                    label += "  (catalog sync)"
                stdscr.addnstr(row + 3, 0, label, max(0, width - 1))
                if row == cursor:
                    stdscr.chgat(row + 3, 0, max(0, width - 1), curses.A_REVERSE)
            stdscr.refresh()
            key = stdscr.getch()
            if key in (ord("q"), ord("Q"), 27):
                return None
            if key in (curses.KEY_UP, ord("k")):
                cursor = max(0, cursor - 1)
            elif key in (curses.KEY_DOWN, ord("j")):
                cursor = min(len(entries) - 1, cursor + 1)
            elif key == ord(" "):
                selected[cursor] = not selected[cursor]
            elif key in (ord("a"), ord("A")):
                selected = [True] * len(entries)
            elif key in (ord("n"), ord("N")):
                selected = [False] * len(entries)
            elif key in (curses.KEY_ENTER, 10, 13):
                return [item for item, include in zip(entries, selected) if include]

    return curses.wrapper(screen)


def main() -> int:
    parser = argparse.ArgumentParser(description="Propagate Cybercore updates to local consumers.")
    parser.add_argument("--apply", action="store_true", help="build, install, regenerate, and restart services")
    parser.add_argument("--consumer", action="append", help="limit execution to named consumers or syncs")
    parser.add_argument("--tui", action="store_true", help="choose consumers interactively before applying")
    args = parser.parse_args()

    if args.tui and not os.environ.get("CYBERCORE_UPDATE_PYTHON_TUI"):
        rust_tui = shutil.which("cybercore-update-tui")
        if rust_tui:
            os.execv(rust_tui, [rust_tui])

    config = json.loads(MANIFEST.read_text(encoding="utf-8"))
    selected = set(args.consumer or [])
    if args.tui:
        if not sys.stdin.isatty() or not sys.stdout.isatty():
            parser.error("--tui requires an interactive terminal")
        chosen = run_tui(config)
        if chosen is None:
            print("Cancelled.")
            return 0
        config = {"consumers": [item for item in chosen if item["kind"] == "consumer"],
                  "syncs": [item for item in chosen if item["kind"] == "sync"]}
        selected = set()
        args.apply = True
    results: list[str] = []
    for item in config["consumers"]:
        if selected and item["name"] not in selected:
            continue
        results.append(process_consumer(item, args.apply))
    for item in config["syncs"]:
        if selected and item["name"] not in selected:
            continue
        results.append(process_sync(item, args.apply))

    print(
        f"SUMMARY ready={results.count('ready')} updated={results.count('updated')} "
        f"blocked={results.count('blocked')}"
    )
    return 1 if "blocked" in results else 0


if __name__ == "__main__":
    raise SystemExit(main())
