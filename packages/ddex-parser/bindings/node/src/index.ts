// bindings/node/src/index.ts
export * from './parser';
export * from './types';

// Re-export the default parser
import DDEXParser from './parser';
export default DDEXParser;