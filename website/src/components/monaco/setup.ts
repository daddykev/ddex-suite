// Monaco from the npm package, bundled by webpack (no CDN). Imported only through the
// dynamic import in ../CodeEditor.tsx, so it lands in an async chunk that only the
// Playground loads, and never runs during the server-side render (docusaurus.config.ts
// also aliases it to an empty module in the server build).
import * as monaco from 'monaco-editor';

// The core editor worker (diffs, links, word-based suggestions). Monaco's JSON support
// creates its own worker with `new Worker(new URL('json.worker.js', import.meta.url))`,
// which webpack bundles without help; the other language workers (TypeScript, CSS, HTML)
// are left out of the build in docusaurus.config.ts.
self.MonacoEnvironment = {
  getWorker: () => new Worker(new URL('./editor.worker.ts', import.meta.url), { type: 'module' }),
};

export { monaco };
