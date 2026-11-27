# DDEX Suite website

The documentation site, [ddex-suite.org](https://ddex-suite.org), built with
[Docusaurus](https://docusaurus.io/), and the Playground API in `functions/` (Cloud
Functions for Firebase).

```bash
npm ci          # Node.js 22 or newer
npm start       # development server
npm run build   # production build in build/ (fails on any broken link or anchor)
```

## Layout

- `docs/`: the documentation pages. The sidebar (`sidebars.ts`) lists every page; do not
  add a page without adding it there.
- `src/pages/index.tsx`: the landing page. Its code tabs show the files in
  `src/examples/` verbatim.
- `src/pages/playground.tsx`: the Playground, which calls the API in `functions/`.
- `docusaurus.config.ts`: site config, including the redirects from the old docs URLs.

## Code examples are tested

Every `js`, `python` and `rust` code block in `docs/`, and each file in `src/examples/`, is
run in CI (job "Website docs examples" in `.github/workflows/ci.yml`) against the packages
built from the repository, with the official sample `examples/Samples43/1 Audio.xml` as
`release.xml`. Blocks that are not meant to run use `bash`, `text`, `xml`, `json` or `toml`;
any other tag fails the check. To run the JS and Python examples locally against installed
packages:

```bash
python3 scripts/readme_examples.py --lang js --workdir /tmp/app --sample "examples/Samples43/1 Audio.xml" --strict website/docs/*.md* website/src/examples/*
```

(`/tmp/app` needs a `node_modules` with `ddex-parser` and `ddex-builder`; for `--lang python`
the running interpreter needs both packages and pandas.)

## Deploy

Deployment is manual (`firebase deploy`) and not part of CI. Run it from `website/` after
`npm ci` here and in `functions/`: deploying hosting first runs `npm run build` (the
`predeploy` in `firebase.json`), so a stale `build/` is never uploaded.
