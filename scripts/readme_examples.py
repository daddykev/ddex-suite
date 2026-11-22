#!/usr/bin/env python3
"""Run the code examples of the READMEs and the website docs against the packages.

Every fenced block whose info string is `js` / `javascript` (with --lang js), `python` /
`py` (with --lang python) or `rust` (with --lang rust) is an example and must run without
error. Blocks that are not meant to run use another info string (`bash`, `text`, `xml`,
...). The document is the only copy of each example, so the document and the test cannot
drift apart. A source file (`.js` / `.cjs`, `.py`, `.rs`) given instead of a document is
one example (the landing page's code tabs are such files).

With --strict, every fenced block must have an info string from RUNNABLE or SKIPPED, so an
example cannot escape the test under a tag the extractor ignores (`ts`, `tsx`, no tag...).
The website docs are checked with --strict.

Each example is written to its own file in WORKDIR and run there, with the sample message
copied to WORKDIR/release.xml, the file name the examples read:
- js: `node file.cjs`; WORKDIR must contain a node_modules with ddex-parser and
  ddex-builder installed.
- python: `python file.py` with the running interpreter, which must have the packages.
- rust: WORKDIR becomes a crate (`docs-examples`) with path dependencies on the crates in
  --repo and a copy of its Cargo.lock; each block is `examples/<name>.rs` (it must have a
  `fn main`), all of them are compiled with one `cargo build --examples`, then each is run
  with `cargo run --example`.

Usage:
  python3 scripts/readme_examples.py --lang js --workdir DIR --sample FILE README.md ...
  python3 scripts/readme_examples.py --lang rust --repo . --workdir DIR --sample FILE --strict website/docs/*.md*
"""

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

LANGS = {
    "js": ({"js", "javascript"}, ".cjs", {".js", ".cjs"}),
    "python": ({"python", "py"}, ".py", {".py"}),
    "rust": ({"rust"}, ".rs", {".rs"}),
}
RUNNABLE = set().union(*(names for names, _, _ in LANGS.values()))
SKIPPED = {"bash", "sh", "shell", "console", "text", "xml", "json", "toml"}
FENCE = re.compile(r"^```([^\n`]*)\n(.*?)^```[ \t]*$", re.S | re.M)

CRATE_MANIFEST = """\
[package]
name = "docs-examples"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
ddex-core = {{ path = "{repo}/packages/core" }}
ddex-parser = {{ path = "{repo}/packages/ddex-parser" }}
ddex-builder = {{ path = "{repo}/packages/ddex-builder" }}

# Not part of an enclosing workspace.
[workspace]
"""


def info_name(info):
    return info.strip().split(",")[0].split(" ")[0].lower()


def blocks(path, names, strict, problems):
    """Yield (line, code) for each example in `path` written in one of `names`."""
    if Path(path).suffix in {".md", ".mdx"}:
        text = Path(path).read_text(encoding="utf-8")
        for match in FENCE.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            info = info_name(match.group(1))
            if strict and info not in RUNNABLE | SKIPPED:
                problems.append(
                    f"{path}:{line}: code block tagged {info or '(no tag)'!r}; use one of "
                    f"{sorted(RUNNABLE)} (run in CI) or {sorted(SKIPPED)} (not run)"
                )
            if info in names:
                yield line, match.group(2)
    else:
        yield 1, Path(path).read_text(encoding="utf-8")


def run(cmd, cwd, timeout=120):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)


def setup_rust(workdir, repo):
    repo = Path(repo).resolve()
    (workdir / "examples").mkdir(parents=True, exist_ok=True)
    for old in (workdir / "examples").glob("*.rs"):
        old.unlink()
    (workdir / "src").mkdir(exist_ok=True)
    (workdir / "src" / "lib.rs").write_text("//! Website docs examples (generated).\n")
    (workdir / "Cargo.toml").write_text(CRATE_MANIFEST.format(repo=repo.as_posix()))
    shutil.copyfile(repo / "Cargo.lock", workdir / "Cargo.lock")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lang", choices=sorted(LANGS), required=True)
    ap.add_argument("--workdir", required=True)
    ap.add_argument("--sample", required=True)
    ap.add_argument("--repo", default=".", help="repository root (rust: the crates and Cargo.lock)")
    ap.add_argument("--strict", action="store_true", help="fail on code blocks with an unknown tag")
    ap.add_argument("docs", nargs="+", help="Markdown/MDX documents or example source files")
    args = ap.parse_args()

    names, suffix, file_suffixes = LANGS[args.lang]
    workdir = Path(args.workdir).resolve()
    workdir.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(args.sample, workdir / "release.xml")
    if args.lang == "rust":
        setup_rust(workdir, args.repo)

    problems, examples = [], []
    for doc in args.docs:
        if Path(doc).suffix not in {".md", ".mdx"} and Path(doc).suffix not in file_suffixes:
            continue  # a source file in another language
        slug = re.sub(r"[^A-Za-z0-9]+", "_", doc).strip("_").lower()
        for line, code in blocks(doc, names, args.strict, problems):
            name = f"{slug}_L{line}"
            if args.lang == "rust":
                if not re.search(r"^\s*fn main\s*\(", code, re.M):
                    problems.append(f"{doc}:{line}: a rust example needs a `fn main`")
                    continue
                script = workdir / "examples" / f"{name}.rs"
            else:
                script = workdir / f"{name}{suffix}"
            script.write_text(code, encoding="utf-8")
            examples.append((f"{doc}:{line}", name, script))

    if problems:
        print("\n".join(problems))
        return 1
    if not examples:
        print(f"no {args.lang} examples found")
        return 1

    if args.lang == "rust":
        build = run(["cargo", "build", "--examples"], workdir, timeout=3600)
        if build.returncode != 0:
            print(build.stdout[-4000:], build.stderr[-20000:], sep="\n")
            print("\nthe rust examples do not compile")
            return 1

    failed = []
    for where, name, script in examples:
        if args.lang == "rust":
            cmd = ["cargo", "run", "--quiet", "--example", name]
        else:
            cmd = (["node"] if args.lang == "js" else [sys.executable]) + [script.name]
        result = run(cmd, workdir)
        status = "ok  " if result.returncode == 0 else "FAIL"
        print(f"{status} {where}")
        if result.returncode != 0:
            failed.append(where)
            print(result.stdout[-2000:], result.stderr[-4000:], sep="\n")

    print(f"\n{len(examples) - len(failed)}/{len(examples)} {args.lang} examples passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
