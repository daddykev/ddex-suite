/* tslint:disable */
/* eslint-disable */
export function main(): void;
export function batchBuild(requests: any): Promise<string[]>;
export function validateStructure(xml: string): ValidationResult;
export function version(): string;
export class BuilderStats {
  free(): void;
  constructor();
  releases_count: number;
  resources_count: number;
  total_build_time_ms: number;
  last_build_size_bytes: number;
  validation_errors: number;
  validation_warnings: number;
}
export class Release {
  free(): void;
  constructor(release_id: string, release_type: string, title: string, artist: string);
  release_id: string;
  release_type: string;
  title: string;
  artist: string;
  get label(): string | undefined;
  set label(value: string | null | undefined);
  get catalog_number(): string | undefined;
  set catalog_number(value: string | null | undefined);
  get upc(): string | undefined;
  set upc(value: string | null | undefined);
  get release_date(): string | undefined;
  set release_date(value: string | null | undefined);
  get genre(): string | undefined;
  set genre(value: string | null | undefined);
  get parental_warning(): boolean | undefined;
  set parental_warning(value: boolean | null | undefined);
  track_ids: string[];
  metadata: any;
}
export class Resource {
  free(): void;
  constructor(resource_id: string, resource_type: string, title: string, artist: string);
  resource_id: string;
  resource_type: string;
  title: string;
  artist: string;
  get isrc(): string | undefined;
  set isrc(value: string | null | undefined);
  get duration(): string | undefined;
  set duration(value: string | null | undefined);
  get track_number(): number | undefined;
  set track_number(value: number | null | undefined);
  get volume_number(): number | undefined;
  set volume_number(value: number | null | undefined);
  metadata: any;
}
export class ValidationResult {
  free(): void;
  constructor(is_valid: boolean);
  is_valid: boolean;
  errors: string[];
  warnings: string[];
}
export class WasmDdexBuilder {
  free(): void;
  constructor();
  addRelease(release: Release): void;
  addResource(resource: Resource): void;
  build(): Promise<string>;
  validate(): ValidationResult;
  getStats(): BuilderStats;
  reset(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly __wbg_release_free: (a: number, b: number) => void;
  readonly __wbg_get_release_release_id: (a: number, b: number) => void;
  readonly __wbg_set_release_release_id: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_release_type: (a: number, b: number) => void;
  readonly __wbg_set_release_release_type: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_title: (a: number, b: number) => void;
  readonly __wbg_set_release_title: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_artist: (a: number, b: number) => void;
  readonly __wbg_set_release_artist: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_label: (a: number, b: number) => void;
  readonly __wbg_set_release_label: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_catalog_number: (a: number, b: number) => void;
  readonly __wbg_set_release_catalog_number: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_upc: (a: number, b: number) => void;
  readonly __wbg_set_release_upc: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_release_date: (a: number, b: number) => void;
  readonly __wbg_set_release_release_date: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_genre: (a: number, b: number) => void;
  readonly __wbg_set_release_genre: (a: number, b: number, c: number) => void;
  readonly __wbg_get_release_parental_warning: (a: number) => number;
  readonly __wbg_set_release_parental_warning: (a: number, b: number) => void;
  readonly release_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly release_track_ids: (a: number, b: number) => void;
  readonly release_set_track_ids: (a: number, b: number, c: number) => void;
  readonly release_metadata: (a: number) => number;
  readonly release_set_metadata: (a: number, b: number, c: number) => void;
  readonly __wbg_resource_free: (a: number, b: number) => void;
  readonly __wbg_get_resource_resource_id: (a: number, b: number) => void;
  readonly __wbg_set_resource_resource_id: (a: number, b: number, c: number) => void;
  readonly __wbg_get_resource_resource_type: (a: number, b: number) => void;
  readonly __wbg_set_resource_resource_type: (a: number, b: number, c: number) => void;
  readonly __wbg_get_resource_title: (a: number, b: number) => void;
  readonly __wbg_set_resource_title: (a: number, b: number, c: number) => void;
  readonly __wbg_get_resource_artist: (a: number, b: number) => void;
  readonly __wbg_set_resource_artist: (a: number, b: number, c: number) => void;
  readonly __wbg_get_resource_isrc: (a: number, b: number) => void;
  readonly __wbg_set_resource_isrc: (a: number, b: number, c: number) => void;
  readonly __wbg_get_resource_duration: (a: number, b: number) => void;
  readonly __wbg_set_resource_duration: (a: number, b: number, c: number) => void;
  readonly __wbg_get_resource_track_number: (a: number) => number;
  readonly __wbg_set_resource_track_number: (a: number, b: number) => void;
  readonly __wbg_get_resource_volume_number: (a: number) => number;
  readonly __wbg_set_resource_volume_number: (a: number, b: number) => void;
  readonly resource_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly resource_metadata: (a: number) => number;
  readonly resource_set_metadata: (a: number, b: number, c: number) => void;
  readonly __wbg_validationresult_free: (a: number, b: number) => void;
  readonly __wbg_get_validationresult_is_valid: (a: number) => number;
  readonly __wbg_set_validationresult_is_valid: (a: number, b: number) => void;
  readonly validationresult_new: (a: number) => number;
  readonly validationresult_errors: (a: number, b: number) => void;
  readonly validationresult_warnings: (a: number, b: number) => void;
  readonly validationresult_set_errors: (a: number, b: number, c: number) => void;
  readonly validationresult_set_warnings: (a: number, b: number, c: number) => void;
  readonly __wbg_builderstats_free: (a: number, b: number) => void;
  readonly __wbg_get_builderstats_releases_count: (a: number) => number;
  readonly __wbg_set_builderstats_releases_count: (a: number, b: number) => void;
  readonly __wbg_get_builderstats_resources_count: (a: number) => number;
  readonly __wbg_set_builderstats_resources_count: (a: number, b: number) => void;
  readonly __wbg_get_builderstats_total_build_time_ms: (a: number) => number;
  readonly __wbg_set_builderstats_total_build_time_ms: (a: number, b: number) => void;
  readonly __wbg_get_builderstats_last_build_size_bytes: (a: number) => number;
  readonly __wbg_set_builderstats_last_build_size_bytes: (a: number, b: number) => void;
  readonly __wbg_get_builderstats_validation_errors: (a: number) => number;
  readonly __wbg_set_builderstats_validation_errors: (a: number, b: number) => void;
  readonly __wbg_get_builderstats_validation_warnings: (a: number) => number;
  readonly __wbg_set_builderstats_validation_warnings: (a: number, b: number) => void;
  readonly builderstats_new: () => number;
  readonly __wbg_wasmddexbuilder_free: (a: number, b: number) => void;
  readonly wasmddexbuilder_new: (a: number) => void;
  readonly wasmddexbuilder_addRelease: (a: number, b: number) => void;
  readonly wasmddexbuilder_addResource: (a: number, b: number) => void;
  readonly wasmddexbuilder_build: (a: number) => number;
  readonly wasmddexbuilder_validate: (a: number) => number;
  readonly wasmddexbuilder_getStats: (a: number) => number;
  readonly wasmddexbuilder_reset: (a: number) => void;
  readonly batchBuild: (a: number) => number;
  readonly validateStructure: (a: number, b: number) => number;
  readonly version: (a: number) => void;
  readonly __wbindgen_export_0: (a: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_2: (a: number, b: number) => number;
  readonly __wbindgen_export_3: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_5: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_6: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
