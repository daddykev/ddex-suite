#!/usr/bin/env python3
"""Checks on the built website (`website/build`), run by CI after `npm run build`.

1. No file refers to a public code CDN (jsDelivr, unpkg, cdnjs, ...): every script and
   style the site runs comes from its own build. (The Playground's Monaco editor used to
   load from cdn.jsdelivr.net at runtime.)
2. Monaco is not in the bundle every page loads (main.js); only the Playground loads it.
3. Report: the JavaScript the Playground page loads, in three groups:
   - shared: runtime~main + main, which every page loads;
   - page: the chunks of the /playground route;
   - editor: the chunks the page imports when the editor mounts (Monaco);
   - on demand: the chunks loaded later (the XML / JSON language modes, the web workers).
   Sizes are bytes, raw and gzip -9 (Firebase Hosting serves compressed responses).

Usage: python3 scripts/website_bundle_check.py website/build   (exit status 1 on a failure)
"""

import gzip
import re
import sys
from pathlib import Path

CDN_HOSTS = [
    "cdn.jsdelivr.net",
    "fastly.jsdelivr.net",
    "unpkg.com",
    "cdnjs.cloudflare.com",
    "esm.sh",
    "cdn.skypack.dev",
    "jspm.io",
    "ajax.googleapis.com",
    "code.jquery.com",
    "cdn.statically.io",
]
TEXT_SUFFIXES = {".html", ".js", ".css", ".json", ".xml", ".txt", ".map", ".svg", ".mjs"}
MONACO_MARKER = "DOMPurify"  # vendored in Monaco's editor core; nowhere else in the site
ROUTE = "/playground"


def sizes(paths):
    raw = sum(p.stat().st_size for p in paths)
    gz = sum(len(gzip.compress(p.read_bytes(), 9)) for p in paths)
    return raw, gz


def chunk_files(js_dir):
    """Map webpack chunk id -> file, from the runtime's chunk filename table."""
    runtime = next(js_dir.glob("runtime~main.*.js")).read_text(encoding="utf-8")
    m = re.search(r'"assets/js/"\+\(\{([^}]*)\}\[\w+\]\|\|\w+\)\+"\."\+\{([^}]*)\}\[\w+\]', runtime)
    if m:
        names = dict(re.findall(r'(\d+):"([^"]+)"', m.group(1)))
        hashes = dict(re.findall(r'(\d+):"([^"]+)"', m.group(2)))
    else:
        m = re.search(r'"assets/js/"\+\w+\+"\."\+\{([^}]*)\}\[\w+\]', runtime)
        names, hashes = {}, dict(re.findall(r'(\d+):"([^"]+)"', m.group(1)))
    files = {}
    for cid, h in hashes.items():
        f = js_dir / f"{names.get(cid, cid)}.{h}.js"
        if f.exists():
            files[cid] = f
    return files


def referenced_chunks(path):
    """Chunk ids a file loads: `x.e(id)` (async import) and `x.u(id)` (worker URL)."""
    text = path.read_text(encoding="utf-8", errors="replace")
    return set(re.findall(r"\b\w\.[eu]\((\d+)\)", text))


def main(build_dir):
    build = Path(build_dir)
    js_dir = build / "assets" / "js"
    failures = 0

    # 1. CDN references
    hits = []
    for path in sorted(build.rglob("*")):
        if path.is_file() and path.suffix in TEXT_SUFFIXES:
            text = path.read_text(encoding="utf-8", errors="replace")
            for host in CDN_HOSTS:
                if host in text:
                    hits.append(f"{path.relative_to(build)}: {host}")
    if hits:
        failures += 1
        print("FAIL: references to public CDNs in the build:")
        for h in hits:
            print("  " + h)
    else:
        print(f"ok   no reference to a public CDN in {len(list(build.rglob('*')))} files")

    # 2. Monaco only on the Playground
    shared = [next(js_dir.glob("runtime~main.*.js")), next(js_dir.glob("main.*.js"))]
    if any(MONACO_MARKER in p.read_text(encoding="utf-8", errors="replace") for p in shared):
        failures += 1
        print("FAIL: Monaco is in the shared bundle (main.js), so every page loads it")
    else:
        print("ok   Monaco is not in the shared bundle")

    # 3. Playground JavaScript
    files = chunk_files(js_dir)
    main_js = shared[1].read_text(encoding="utf-8")
    route = re.search(r'"' + re.escape(ROUTE) + r'-[0-9a-f]+":\{"__comp":"([^"]+)"', main_js)
    entry = route and re.search(r'"' + route.group(1) + r'":\[\(\)=>(.*?)\.then', main_js)
    if not entry:
        print(f"FAIL: could not find the {ROUTE} route chunks in main.js")
        return 1
    page = set(re.findall(r"\b\w\.e\((\d+)\)", entry.group(1)))
    editor = set().union(*(referenced_chunks(files[c]) for c in page if c in files)) - page
    seen, todo = page | editor, list(editor)
    on_demand = set()
    while todo:
        current = todo.pop()
        for c in referenced_chunks(files[current]) if current in files else []:
            if c not in seen:
                seen.add(c)
                on_demand.add(c)
                todo.append(c)
    groups = [
        ("shared (runtime~main + main, every page)", shared),
        (f"page ({ROUTE} route chunks)", [files[c] for c in sorted(page) if c in files]),
        ("editor (imported when the editor mounts)", [files[c] for c in sorted(editor) if c in files]),
        ("on demand (language modes, workers)", [files[c] for c in sorted(on_demand) if c in files]),
    ]
    print(f"\nJavaScript loaded by {ROUTE} (bytes):")
    print(f"  {'group':44} {'files':>5} {'raw':>11} {'gzip':>10}")
    total_raw = total_gz = 0
    for label, paths in groups:
        raw, gz = sizes(paths)
        total_raw += raw
        total_gz += gz
        print(f"  {label:44} {len(paths):>5} {raw:>11,} {gz:>10,}")
    print(f"  {'total':44} {'':>5} {total_raw:>11,} {total_gz:>10,}")
    workers = [p for p in js_dir.glob("*.js") if "worker" in p.name]
    if workers:
        print("  worker files: " + ", ".join(f"{p.name} {p.stat().st_size:,}" for p in sorted(workers)))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "website/build"))
