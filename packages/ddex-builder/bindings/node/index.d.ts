export interface BuildRequest {
  releases: FlattenedRelease[];
  messageHeader?: MessageHeader;
  parties?: Party[];
  deals?: Deal[];
  profile?: string;
  version?: string;
}

export interface BuildOptions {
  determinism?: DeterminismConfig;
  preflightLevel?: 'strict' | 'warn' | 'none';
  validateReferences?: boolean;
  requireMinimumFields?: boolean;
  idStrategy?: 'uuid' | 'uuidv7' | 'sequential' | 'stable-hash';
  stableHashConfig?: StableHashConfig;
  partnerPreset?: string;
}

export interface BuildResult {
  xml: string;
  warnings: BuildWarning[];
  errors: BuildError[];
  statistics: BuildStatistics;
  canonicalHash?: string;
}

export interface PreflightResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  suggestions: Suggestion[];
}

export class DDEXBuilder {
  constructor();
  
  build(request: BuildRequest, options?: BuildOptions): Promise<BuildResult>;
  buildSync(request: BuildRequest, options?: BuildOptions): BuildResult;
  preflight(request: BuildRequest): Promise<PreflightResult>;
  canonicalize(xml: string): string;
  
  // Partner preset methods
  applyPreset(preset: string, options?: PresetOptions): void;
  getAvailablePresets(): string[];
  
  // Debugging methods
  dryRunId(type: string, materials: any, recipe?: string): IdDebugInfo;
}

export default DDEXBuilder;