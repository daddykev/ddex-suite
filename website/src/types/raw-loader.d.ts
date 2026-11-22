// `import source from '!!raw-loader!./file'` imports a file's text (the landing page shows
// the example files in src/examples, which CI runs).
declare module '!!raw-loader!*' {
  const content: string;
  export default content;
}
