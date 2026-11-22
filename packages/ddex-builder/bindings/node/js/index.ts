/**
 * @fileoverview DDEX Builder Node.js API - High-performance DDEX XML generation
 * @module ddex-builder
 * @version 0.5.0
 * @author Kevin Marques Moo
 * @license MIT
 */

// bindings/node/js/index.ts
import {
  DdexBuilder as NativeDdexBuilder,
  StreamingDdexBuilder as NativeStreamingDdexBuilder,
  validateStructure,
  type Release,
  type Resource,
  type ValidationResult,
  type BuilderStats,
  type PresetInfo,
  type ValidationRule,
  type FidelityOptions,
  type BuildStatistics,
  type VerificationResult,
  type BuildResult,
  type FidelityInfo,
  type StreamingConfig,
  type StreamingProgress,
  type StreamingStats,
  type MessageHeader
} from '../index.js';
import type { BuildRequest } from '../build-request';

/**
 * Configuration options for DDEX XML building operations
 * @category Builder Classes
 */
export interface BuildOptions {
  /** DDEX version to target for XML generation */
  version?: '3.8.2' | '4.2' | '4.3';
  /** Industry preset to apply (e.g., 'audio_album', 'audio_single', 'youtube') */
  preset?: string;
  /** Enable deterministic element and attribute ordering */
  enableDeterministicOrdering?: boolean;
  /** Enable perfect round-trip fidelity preservation */
  enablePerfectFidelity?: boolean;
  /** Canonicalization algorithm to use for XML normalization */
  canonicalization?: 'db-c14n-1.0' | 'exclusive' | 'none';
  /** Validate structure during build process */
  validateOnBuild?: boolean;
  /** Include detailed build statistics in output */
  includeStatistics?: boolean;
}

/**
 * Configuration options for streaming DDEX XML operations
 * @category Streaming
 */
export interface StreamingBuildOptions {
  /** Maximum buffer size in bytes before flushing to output */
  maxBufferSize?: number;
  /** Enable deterministic ordering for streaming output */
  deterministic?: boolean;
  /** Validate XML structure during streaming */
  validateDuringStream?: boolean;
  /** Frequency of progress callbacks (number of elements) */
  progressCallbackFrequency?: number;
  /** DDEX version to target for streaming output */
  version?: '3.8.2' | '4.2' | '4.3';
  /** Industry preset to apply to streaming output */
  preset?: string;
}

/**
 * High-performance DDEX XML builder with deterministic output and industry preset support
 *
 * The DDEXBuilder provides a comprehensive API for generating DDEX-compliant XML documents
 * with deterministic output, perfect round-trip fidelity, and support for industry presets.
 *
 * @example
 * ```typescript
 * import { DDEXBuilder } from 'ddex-builder';
 *
 * const builder = new DDEXBuilder();
 *
 * // Apply industry preset
 * builder.applyPreset('audio_album');
 *
 * // Add releases and resources (ERN 4.x needs label, genre, parentalWarningType, and
 * // per resource ddexType and parentalWarningType; a missing one fails the build)
 * builder.addRelease({
 *   releaseId: 'A1-ABCDE-0000000001-M',
 *   releaseType: 'Album',
 *   title: 'My Album',
 *   artist: 'Artist Name',
 *   label: 'My Label',
 *   genre: 'Pop',
 *   parentalWarningType: 'NotExplicit',
 *   trackIds: ['A1', 'A2']
 * });
 *
 * // Build XML: the request carries the header and version (nothing is filled in)
 * const xml = await builder.build({
 *   version: '4.3',
 *   language_and_script_code: 'en',
 *   avs_version_id: '12',
 *   header: {
 *     message_sender: { party_name: [{ text: 'My Label' }], party_id: 'PADPIDA0000000001' },
 *     message_recipient: { party_name: [{ text: 'My DSP' }], party_id: 'PADPIDA0000000002' }
 *   }
 * });
 * console.log(xml);
 * ```
 *
 * @category Builder Classes
 */
export class DDEXBuilder {
  private native: NativeDdexBuilder;

  /**
   * Creates a new DDEXBuilder instance with default configuration
   */
  constructor() {
    this.native = new NativeDdexBuilder();
  }

  /**
   * Add a release to the builder's internal state
   *
   * @param release - Release metadata including title, artist, and track references
   * @throws {Error} When release data is invalid or missing required fields
   *
   * @example
   * ```typescript
   * builder.addRelease({
   *   releaseId: 'REL001',
   *   releaseType: 'Album',
   *   title: 'Greatest Hits',
   *   artist: 'Famous Artist',
   *   label: 'Record Label',
   *   upc: '123456789012',
   *   releaseDate: '2024-01-15',
   *   trackIds: ['TRK001', 'TRK002']
   * });
   * ```
   */
  addRelease(release: Release): void {
    return this.native.addRelease(release);
  }

  /**
   * Add a resource (track/audio file) to the builder's internal state
   *
   * @param resource - Resource metadata including title, artist, and technical details
   * @throws {Error} When resource data is invalid or missing required fields
   *
   * @example
   * ```typescript
   * builder.addResource({
   *   resourceId: 'TRK001',
   *   resourceType: 'SoundRecording',
   *   title: 'Track Title',
   *   artist: 'Artist Name',
   *   isrc: 'USRC17607839',
   *   duration: 'PT3M45S',
   *   trackNumber: 1
   * });
   * ```
   */
  addResource(resource: Resource): void {
    return this.native.addResource(resource);
  }

  /**
   * Build DDEX XML from the current builder state with standard options
   *
   * @param data - Build request (core `BuildRequest` fields, see build-request.d.ts);
   *   `header` and `version` are required. Without `releases`, the releases added with
   *   `addRelease`/`addResource` are used.
   * @param options - Build configuration options
   * @returns Promise resolving to XML string
   * @throws {Error} When build fails due to validation errors or missing data
   *
   * @example
   * ```typescript
   * const xml = await builder.build(request, { preset: 'audio_album' });
   * ```
   */
  async build(data?: BuildRequest | null, options?: BuildOptions): Promise<string> {
    if (options?.preset) {
      this.native.applyPreset(options.preset);
    }
    return this.native.build(data);
  }

  /**
   * Build DDEX XML with full fidelity control and advanced options
   *
   * @param data - Optional additional data to include in build
   * @param fidelityOptions - Advanced fidelity and canonicalization options
   * @returns Promise resolving to BuildResult with XML, statistics, and verification data
   * @throws {Error} When build fails or fidelity requirements cannot be met
   *
   * @example
   * ```typescript
   * const result = await builder.buildWithFidelity(null, {
   *   enablePerfectFidelity: true,
   *   canonicalization: 'db-c14n-1.0',
   *   preserveComments: true,
   *   enableVerification: true,
   *   collectStatistics: true
   * });
   *
   * console.log('XML:', result.xml);
   * console.log('Build time:', result.statistics?.buildTimeMs, 'ms');
   * console.log('Fidelity score:', result.verification?.fidelityScore);
   * ```
   */
  async buildWithFidelity(data?: BuildRequest | null, fidelityOptions?: FidelityOptions): Promise<BuildResult> {
    return this.native.buildWithFidelity(data, fidelityOptions);
  }

  /**
   * Test round-trip fidelity by parsing and rebuilding original XML
   *
   * @param originalXml - Original DDEX XML to test against
   * @param fidelityOptions - Fidelity options for the test
   * @returns Promise resolving to verification results with fidelity score
   * @throws {Error} When round-trip test fails or XML is invalid
   *
   * @example
   * ```typescript
   * const verification = await builder.testRoundTripFidelity(originalXml, {
   *   enablePerfectFidelity: true,
   *   canonicalization: 'db-c14n-1.0'
   * });
   *
   * if (verification.roundTripSuccess) {
   *   console.log('Fidelity score:', verification.fidelityScore);
   * } else {
   *   console.log('Issues:', verification.issues);
   * }
   * ```
   */
  async testRoundTripFidelity(originalXml: string, fidelityOptions?: FidelityOptions): Promise<VerificationResult> {
    return this.native.testRoundTripFidelity(originalXml, fidelityOptions);
  }

  /**
   * Validate the current builder state against DDEX rules and constraints
   *
   * @returns Promise resolving to validation results with errors and warnings
   *
   * @example
   * ```typescript
   * const validation = await builder.validate();
   *
   * if (validation.isValid) {
   *   console.log('Builder state is valid');
   * } else {
   *   console.log('Errors:', validation.errors);
   *   console.log('Warnings:', validation.warnings);
   * }
   * ```
   */
  async validate(): Promise<ValidationResult> {
    return this.native.validate();
  }

  /**
   * Get current builder statistics including counts and performance metrics
   *
   * @returns Builder statistics object
   *
   * @example
   * ```typescript
   * const stats = builder.getStats();
   * console.log('Releases:', stats.releasesCount);
   * console.log('Resources:', stats.resourcesCount);
   * console.log('Last build time:', stats.totalBuildTimeMs, 'ms');
   * ```
   */
  getStats(): BuilderStats {
    return this.native.getStats();
  }

  /**
   * Reset the builder to initial state, clearing all releases and resources
   *
   * @example
   * ```typescript
   * builder.reset();
   * console.log('Builder reset, ready for new data');
   * ```
   */
  reset(): void {
    return this.native.reset();
  }

  /**
   * Get list of available industry presets
   *
   * @returns Array of preset names
   *
   * @example
   * ```typescript
   * const presets = builder.getAvailablePresets();
   * console.log('Available presets:', presets);
   * // Output: ['audio_album', 'audio_single', 'video_single', 'youtube']
   * ```
   */
  getAvailablePresets(): string[] {
    return this.native.getAvailablePresets();
  }

  /**
   * Get detailed information about a specific preset
   *
   * @param presetName - Name of the preset to query
   * @returns Preset information including description and required fields
   * @throws {Error} When preset name is not found
   *
   * @example
   * ```typescript
   * const info = builder.getPresetInfo('audio_album');
   * console.log('Description:', info.description);
   * console.log('Required fields:', info.requiredFields);
   * console.log('Version:', info.version);
   * ```
   */
  getPresetInfo(presetName: string): PresetInfo {
    return this.native.getPresetInfo(presetName);
  }

  /**
   * Apply an industry preset to configure the builder for specific use cases
   *
   * @param presetName - Name of the preset to apply
   * @throws {Error} When preset name is not found or cannot be applied
   *
   * @example
   * ```typescript
   * // Apply YouTube preset for video distribution
   * builder.applyPreset('youtube');
   *
   * // Apply audio album preset for traditional music releases
   * builder.applyPreset('audio_album');
   * ```
   */
  applyPreset(presetName: string): void {
    return this.native.applyPreset(presetName);
  }

  /**
   * Get validation rules associated with a specific preset
   *
   * @param presetName - Name of the preset to query
   * @returns Array of validation rules for the preset
   * @throws {Error} When preset name is not found
   *
   * @example
   * ```typescript
   * const rules = builder.getPresetValidationRules('audio_album');
   * rules.forEach(rule => {
   *   console.log(`${rule.fieldName}: ${rule.message}`);
   * });
   * ```
   */
  getPresetValidationRules(presetName: string): ValidationRule[] {
    return this.native.getPresetValidationRules(presetName);
  }
}

/**
 * Streaming DDEX XML builder for large datasets and real-time processing
 *
 * The StreamingDDEXBuilder is optimized for generating large DDEX documents efficiently
 * by streaming data as it's processed rather than building everything in memory.
 * Ideal for catalog ingestion, bulk uploads, and real-time feed processing.
 *
 * @example
 * ```typescript
 * import { StreamingDDEXBuilder } from 'ddex-builder';
 *
 * const builder = new StreamingDDEXBuilder({
 *   maxBufferSize: 1024 * 1024, // 1MB buffer
 *   deterministic: true,
 *   validateDuringStream: true
 * });
 *
 * // Set up progress tracking
 * builder.setProgressCallback((progress) => {
 *   console.log(`Progress: ${progress.estimatedCompletionPercent}%`);
 * });
 *
 * // Start message
 * builder.startMessage({
 *   messageSenderName: 'My Label',
 *   messageRecipientName: 'Distribution Platform'
 * }, '4.3');
 *
 * // Stream resources and releases
 * const resId = builder.writeResource('RES001', 'Track 1', 'Artist', 'USRC17607839');
 * builder.finishResourcesStartReleases();
 * builder.writeRelease('REL001', 'Album', 'Artist', 'Label', null, null, null, [resId]);
 *
 * // Get final result
 * const stats = builder.finishMessage();
 * const xml = builder.getXml();
 * ```
 *
 * @category Streaming
 */
export class StreamingDDEXBuilder {
  private native: NativeStreamingDdexBuilder;

  /**
   * Creates a new StreamingDDEXBuilder with optional configuration
   *
   * @param config - Streaming configuration options
   */
  constructor(config?: StreamingConfig) {
    this.native = new NativeStreamingDdexBuilder(config);
  }

  /**
   * Set callback function to receive progress updates during streaming operations
   *
   * @param callback - Function called with progress information
   *
   * @example
   * ```typescript
   * builder.setProgressCallback((progress) => {
   *   console.log(`Written: ${progress.bytesWritten} bytes`);
   *   console.log(`Releases: ${progress.releasesWritten}`);
   *   console.log(`Memory usage: ${progress.currentMemoryUsage} bytes`);
   *   if (progress.estimatedCompletionPercent) {
   *     console.log(`Progress: ${progress.estimatedCompletionPercent}%`);
   *   }
   * });
   * ```
   */
  setProgressCallback(callback: (progress: StreamingProgress) => void): void {
    return this.native.setProgressCallback(callback);
  }

  /**
   * Set estimated total number of items for accurate progress tracking
   *
   * @param total - Estimated total number of releases or resources to process
   *
   * @example
   * ```typescript
   * // If you know you'll be processing 1000 releases
   * builder.setEstimatedTotal(1000);
   * ```
   */
  setEstimatedTotal(total: number): void {
    return this.native.setEstimatedTotal(total);
  }

  /**
   * Start a new DDEX message with header information
   *
   * @param header - Message header with sender/recipient information
   * @param version - DDEX version to use for the message
   * @throws {Error} When header is invalid or version is not supported
   *
   * @example
   * ```typescript
   * builder.startMessage({
   *   messageId: 'MSG001',
   *   messageSenderName: 'My Music Label',
   *   messageRecipientName: 'Spotify',
   *   messageCreatedDateTime: '2024-01-15T10:30:00Z'
   * }, '4.3');
   * ```
   */
  startMessage(header: MessageHeader, version: string): void {
    return this.native.startMessage(header, version);
  }

  /**
   * Write a resource (track/audio file) to the streaming output
   *
   * @param resourceId - Unique identifier for the resource
   * @param title - Resource title
   * @param artist - Primary artist name
   * @param isrc - International Standard Recording Code (optional)
   * @param duration - Track duration in ISO 8601 format (optional)
   * @param filePath - Path to audio file (optional)
   * @returns Generated internal resource reference ID
   * @throws {Error} When resource data is invalid
   *
   * @example
   * ```typescript
   * const resourceRef = builder.writeResource(
   *   'TRK001',
   *   'Beautiful Song',
   *   'Amazing Artist',
   *   'USRC17607839',
   *   'PT3M45S',
   *   '/path/to/audio.mp3'
   * );
   * ```
   */
  writeResource(
    resourceId: string,
    title: string,
    artist: string,
    isrc?: string,
    duration?: string,
    filePath?: string
  ): string {
    return this.native.writeResource(resourceId, title, artist, isrc, duration, filePath);
  }

  /**
   * Complete the resources section and begin the releases section
   *
   * Must be called after all resources are written and before writing releases.
   *
   * @throws {Error} When called at the wrong time in the streaming process
   *
   * @example
   * ```typescript
   * // Write all resources first
   * const res1 = builder.writeResource('RES001', 'Track 1', 'Artist');
   * const res2 = builder.writeResource('RES002', 'Track 2', 'Artist');
   *
   * // Then transition to releases
   * builder.finishResourcesStartReleases();
   *
   * // Now write releases
   * builder.writeRelease('REL001', 'Album', 'Artist', 'Label', null, null, null, [res1, res2]);
   * ```
   */
  finishResourcesStartReleases(): void {
    return this.native.finishResourcesStartReleases();
  }

  /**
   * Write a release to the streaming output
   *
   * @param releaseId - Unique identifier for the release
   * @param title - Release title
   * @param artist - Primary artist name
   * @param label - Record label name
   * @param upc - Universal Product Code
   * @param releaseDate - Release date in ISO format
   * @param genre - Primary genre
   * @param resourceReferences - Array of resource reference IDs from writeResource calls
   * @returns Generated internal release reference ID
   * @throws {Error} When release data is invalid or resources don't exist
   *
   * @example
   * ```typescript
   * const releaseRef = builder.writeRelease(
   *   'REL001',
   *   'Greatest Hits Album',
   *   'Famous Artist',
   *   'My Record Label',
   *   '123456789012',
   *   '2024-03-15',
   *   'Pop',
   *   [resourceRef1, resourceRef2, resourceRef3]
   * );
   * ```
   */
  writeRelease(
    releaseId: string,
    title: string,
    artist: string,
    label: string | null,
    upc: string | null,
    releaseDate: string | null,
    genre: string | null,
    resourceReferences: string[]
  ): string {
    return this.native.writeRelease(releaseId, title, artist, label, upc, releaseDate, genre, resourceReferences);
  }

  /**
   * Complete the DDEX message and get final processing statistics
   *
   * @returns Final statistics including counts, memory usage, and any warnings
   * @throws {Error} When message cannot be completed due to validation errors
   *
   * @example
   * ```typescript
   * const stats = builder.finishMessage();
   * console.log(`Processed ${stats.releasesWritten} releases`);
   * console.log(`Processed ${stats.resourcesWritten} resources`);
   * console.log(`Final size: ${stats.bytesWritten} bytes`);
   * console.log(`Peak memory: ${stats.peakMemoryUsage} bytes`);
   *
   * if (stats.warnings.length > 0) {
   *   console.log('Warnings:', stats.warnings);
   * }
   * ```
   */
  finishMessage(): StreamingStats {
    return this.native.finishMessage();
  }

  /**
   * Get the complete generated XML document
   *
   * @returns Complete DDEX XML document as string
   * @throws {Error} When called before finishMessage() or if generation failed
   *
   * @example
   * ```typescript
   * // After finishing the message
   * const stats = builder.finishMessage();
   * const xml = builder.getXml();
   *
   * // Save to file or send to API
   * await fs.writeFile('catalog.xml', xml);
   * ```
   */
  getXml(): string {
    return this.native.getXml();
  }

  /**
   * Reset the streaming builder to initial state for reuse
   *
   * @example
   * ```typescript
   * // Process first batch
   * builder.startMessage(header1, '4.3');
   * // ... write data ...
   * const xml1 = builder.getXml();
   *
   * // Reset and process second batch
   * builder.reset();
   * builder.startMessage(header2, '4.3');
   * // ... write different data ...
   * const xml2 = builder.getXml();
   * ```
   */
  reset(): void {
    return this.native.reset();
  }
}

/**
 * Validate DDEX XML structure without building a complete document
 *
 * Performs structural validation of DDEX XML against schema rules and constraints.
 * Useful for validating existing XML documents or checking partial builds.
 *
 * @param xml - DDEX XML document to validate
 * @returns Promise resolving to validation results with detailed error information
 *
 * @example
 * ```typescript
 * import { validateDDEXStructure } from 'ddex-builder';
 *
 * const xml = `<?xml version="1.0"?>
 * <ernm:NewReleaseMessage xmlns:ernm="http://ddex.net/xml/ern/43">
 *   <!-- DDEX content -->
 * </ernm:NewReleaseMessage>`;
 *
 * try {
 *   const validation = await validateDDEXStructure(xml);
 *
 *   if (validation.isValid) {
 *     console.log('✅ XML is valid DDEX structure');
 *   } else {
 *     console.log('❌ Validation failed:');
 *     validation.errors.forEach(error => console.log(`  - ${error}`));
 *   }
 *
 *   if (validation.warnings.length > 0) {
 *     console.log('⚠️  Warnings:');
 *     validation.warnings.forEach(warning => console.log(`  - ${warning}`));
 *   }
 * } catch (error) {
 *   console.error('Validation error:', error);
 * }
 * ```
 *
 * @category Validation
 */
export async function validateDDEXStructure(xml: string): Promise<ValidationResult> {
  return validateStructure(xml);
}

// Build request types (build-request.d.ts)
export type {
  BuildRequest,
  AvsValue,
  LocalizedString,
  Party,
  MessageHeader as BuildMessageHeader,
  ProprietaryId,
  ReleaseIds,
  ResourceGroup,
  TrackRelease,
  Track,
  Release as BuildRelease,
  Deal,
} from '../build-request';

// Re-export all types from the generated index.d.ts
export type {
  Release,
  Resource,
  ValidationResult,
  BuilderStats,
  PresetInfo,
  ValidationRule,
  FidelityOptions,
  BuildStatistics,
  VerificationResult,
  BuildResult,
  FidelityInfo,
  StreamingConfig,
  StreamingProgress,
  StreamingStats,
  MessageHeader
};