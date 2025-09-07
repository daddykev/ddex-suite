// bindings/node/src/types.ts
/**
 * Type definitions for DDEX Parser
 * These types will be auto-generated from Rust
 */

export interface LocalizedString {
  text: string;
  languageCode?: string;
  script?: string;
}

export interface Identifier {
  type: 'Proprietary' | 'ISRC' | 'ISWC' | 'UPC' | 'EAN' | 'GRID' | 'GRid' | 'ISNI' | 'IPI';
  namespace?: string;
  value: string;
}

export interface Copyright {
  text: string;
  year?: number;
  owner?: string;
}

export interface Money {
  amount: number;
  currency: string;
}

// Placeholder for auto-generated types
export interface ERNMessage {
  // Will be generated from Rust
  [key: string]: any;
}

export interface Release {
  // Will be generated from Rust
  [key: string]: any;
}

export interface Resource {
  // Will be generated from Rust
  [key: string]: any;
}

export interface Deal {
  // Will be generated from Rust
  [key: string]: any;
}

export interface Party {
  // Will be generated from Rust
  [key: string]: any;
}