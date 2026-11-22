#!/usr/bin/env python3
"""Run the code examples of the READMEs against the installed packages.

Every fenced block whose info string is `js` / `javascript` (with --lang js) or
`python` / `py` (with --lang python) is an example and must run without error. Blocks
that are not meant to run use another info string (`ts`, `text`, `bash`, ...). The README
is the only copy of each example, so the README and the test cannot drift apart.

Each block is written to its own file in WORKDIR and run there (`node file.cjs` /
`python file.py`), with the sample message copied to WORKDIR/release.xml, the file name
the examples read. For js, WORKDIR must contain a node_modules with ddex-parser and
ddex-builder installed; for python, they must be installed in the running interpreter.

Usage:
  python3 scripts/readme_examples.py --lang js --workdir DIR --sample FILE README.md ...
"""

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

LANGS = {"js": ({"js", "javascript"}, ".cjs"), "python": ({"python", "py"}, ".py")}
FENCE = re.compile(r"^```([^\n`]*)\n(.*?)^```[ \t]*$", re.S | re.M)


def blocks(readme, names):
    text = Path(readme).read_text(encoding="utf-8")
    for match in FENCE.finditer(text):
        info = match.group(1).strip().split(",")[0].split(" ")[0].lower()
        if info in names:
            line = text.count("\n", 0, match.start()) + 1
            yield line, match.group(2)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lang", choices=sorted(LANGS), required=True)
    ap.add_argument("--workdir", required=True)
    ap.add_argument("--sample", required=True)
    ap.add_argument("readmes", nargs="+")
    args = ap.parse_args()

    names, suffix = LANGS[args.lang]
    workdir = Path(args.workdir).resolve()
    workdir.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(args.sample, workdir / "release.xml")
    runner = ["node"] if args.lang == "js" else [sys.executable]

    ran, failed = 0, []
    for readme in args.readmes:
        slug = re.sub(r"[^A-Za-z0-9]+", "_", readme).strip("_")
        for line, code in blocks(readme, names):
            script = workdir / f"{slug}_L{line}{suffix}"
            script.write_text(code, encoding="utf-8")
            result = subprocess.run(
                runner + [script.name], cwd=workdir, capture_output=True, text=True, timeout=120
            )
            ran += 1
            status = "ok  " if result.returncode == 0 else "FAIL"
            print(f"{status} {readme}:{line}")
            if result.returncode != 0:
                failed.append(f"{readme}:{line}")
                print(result.stdout[-2000:], result.stderr[-4000:], sep="\n")

    if ran == 0:
        print(f"no {args.lang} examples found")
        return 1
    print(f"\n{ran - len(failed)}/{ran} {args.lang} README examples passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
