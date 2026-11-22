#!/usr/bin/env python3
"""Check the contents of the npm tarballs made by `npm pack` for a release.

Each tarball must contain exactly the expected files, so nothing stale, generated or
private is published, and nothing needed is missing:
- ddex-parser:  package.json README.md LICENSE index.js index.d.ts
- ddex-builder: the same plus build-request.d.ts
- <package>-<platform>: package.json README.md and the native binary named in "main"
Each package.json must carry the release version, and a platform package's binary must
not be empty.

Usage: python3 scripts/check_npm_packs.py DIR_WITH_TGZ [--version X.Y.Z]
"""

import argparse
import json
import sys
import tarfile
from pathlib import Path

MAIN = {
    "ddex-parser": {"package.json", "README.md", "LICENSE", "index.js", "index.d.ts"},
    "ddex-builder": {"package.json", "README.md", "LICENSE", "index.js", "index.d.ts", "build-request.d.ts"},
}
PLATFORMS = {"darwin-arm64", "darwin-x64", "linux-x64-gnu", "win32-x64-msvc"}


def check(path, version):
    errors = []
    with tarfile.open(path) as tar:
        members = {m.name.split("/", 1)[1]: m for m in tar.getmembers() if m.isfile()}
        manifest = json.load(tar.extractfile(members["package.json"]))
    name = manifest["name"]
    if version and manifest.get("version") != version:
        errors.append(f"version {manifest.get('version')} != {version}")
    if name in MAIN:
        expected = MAIN[name]
    else:
        pkg = next((p for p in MAIN if name.startswith(p + "-")), None)
        platform = name[len(pkg) + 1:] if pkg else None
        if pkg is None or platform not in PLATFORMS:
            return name, [f"unexpected package {name}"]
        binary = manifest.get("main", "")
        expected = {"package.json", "README.md", binary}
        if binary in members and members[binary].size == 0:
            errors.append(f"{binary} is empty")
    files = set(members)
    if files - expected:
        errors.append(f"unexpected files: {sorted(files - expected)}")
    if expected - files:
        errors.append(f"missing files: {sorted(expected - files)}")
    return name, errors


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--version")
    args = ap.parse_args()
    tarballs = sorted(Path(args.dir).glob("*.tgz"))
    if not tarballs:
        print(f"no .tgz in {args.dir}")
        return 1
    failures = 0
    for path in tarballs:
        name, errors = check(path, args.version)
        print(f"{'FAIL' if errors else 'ok  '} {path.name} ({name})")
        for error in errors:
            print(f"     {error}")
        failures += bool(errors)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
