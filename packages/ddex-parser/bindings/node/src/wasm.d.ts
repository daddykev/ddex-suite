// packages/ddex-parser/bindings/node/src/wasm.d.ts
declare module '../wasm/ddex_parser' {
  export class DDEXParser {
    constructor();
    parse(xml: string, options: any): any;
    parse_stream(stream: any, options: any): Promise<any>;
    version(): string;
  }
  
  export default function init(): Promise<void>;
}

// Also declare module for the actual path when WASM is built
declare module '../../wasm/pkg/ddex_parser' {
  export class DDEXParser {
    constructor();
    parse(xml: string, options: any): any;
    parse_stream(stream: any, options: any): Promise<any>;
    version(): string;
  }
  
  export default function init(): Promise<void>;
}