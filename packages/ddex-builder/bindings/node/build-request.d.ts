/**
 * The build request accepted by `DdexBuilder.build(request)` / `buildWithFidelity(request)`.
 *
 * It mirrors the Rust core `BuildRequest` (field names are the core's snake_case names) and
 * is passed to the core builder unchanged. Nothing is filled in: when the target ERN
 * version's schema requires a value the request does not carry, the build fails with an
 * error naming the field, e.g.
 * `ERN 4.3 requires Release/ReleaseType (releases[0].release_type)`.
 *
 * Fields marked "ERN 4.x" are only written for 4.2/4.3 targets.
 */

/** A DDEX allowed-value-set term, or `{ value: "UserDefined", user_defined_value: ... }`. */
export type AvsValue =
  | string
  | {
      value: string
      /** `@Namespace`: the party that defined a user-defined value */
      namespace?: string | null
      /** `@UserDefinedValue`, used with `value: "UserDefined"` */
      user_defined_value?: string | null
    }

export interface LocalizedString {
  text: string
  language_code?: string | null
}

export interface Party {
  party_name: LocalizedString[]
  /** ERN 4.x: required for sender/recipient, a DDEX Party ID (`PADPIDA...`) */
  party_id?: string | null
  party_reference?: string | null
}

export interface MessageHeader {
  /** Generated when omitted */
  message_id?: string | null
  message_sender: Party
  message_recipient: Party
  /** `LiveMessage` or `TestMessage` */
  message_control_type?: string | null
  /** ISO 8601 date-time; the current time when omitted */
  message_created_date_time?: string | null
}

export interface ProprietaryId {
  namespace: string
  value: string
}

/** ERN 4.x: all identifiers of a release, written as one `ReleaseId`. */
export interface ReleaseIds {
  grid?: string | null
  icpn?: string | null
  proprietary_ids?: ProprietaryId[]
}

/** ERN 4.x: a `ResourceGroup` node (nested groups need `resource_group_type`). */
export interface ResourceGroup {
  resource_group_type?: string | null
  sequence_number?: number | null
  resource_groups?: ResourceGroup[]
  content_items?: Array<{ sequence_number?: number | null; resource_reference: string }>
}

/** ERN 4.x: a `TrackRelease` for one track. */
export interface TrackRelease {
  release_reference: string
  release_ids: ReleaseIds
  title?: string | null
  genre?: string | null
  sub_genre?: string | null
  label?: string | null
}

export interface Track {
  track_id: string
  resource_reference?: string | null
  isrc: string
  title: string
  /** ISO 8601 duration, e.g. `PT3M45S` */
  duration: string
  artist: string
  /** ERN 4.x: `SoundRecording` (default) or `Video` */
  resource_kind?: 'SoundRecording' | 'Video'
  /** ERN 4.x (required): the resource's DDEX `Type`, e.g. `MusicalWorkSoundRecording` */
  resource_type?: AvsValue | null
  /** ERN 4.x (required): `ParentalWarningType`, e.g. `NotExplicit` */
  parental_warning_type?: string | null
  /** ERN 4.x: `DisplayArtistRole` of `artist` (default `MainArtist`) */
  artist_role?: AvsValue | null
  disc_number?: number | null
  sequence_number?: number | null
  proprietary_ids?: ProprietaryId[]
  track_release?: TrackRelease | null
}

export interface Release {
  /** Written as `ReleaseId/GRid` unless `release_ids` is set (ERN 4.x) */
  release_id: string
  release_reference?: string | null
  title: LocalizedString[]
  /** Main artist (ERN 4.x: `DisplayArtistName` and a `MainArtist` `DisplayArtist`) */
  artist: string
  /** ERN 4.x (required): label name, referenced by `ReleaseLabelReference` */
  label?: string | null
  release_date?: string | null
  upc?: string | null
  tracks: Track[]
  resource_references?: string[] | null
  /** ERN 4.x (required), e.g. `Album` */
  release_type?: AvsValue | null
  /** ERN 4.x (required) */
  genre?: string | null
  sub_genre?: string | null
  /** ERN 4.x (required), e.g. `NotExplicit` */
  parental_warning_type?: string | null
  artist_role?: AvsValue | null
  release_ids?: ReleaseIds | null
  resource_group?: ResourceGroup | null
}

export interface Deal {
  deal_reference?: string | null
  deal_terms: {
    /** ERN 4.x: omitted when empty */
    commercial_model_type: string
    territory_code: string[]
    /** YYYY-MM-DD (ERN 4.x: required) */
    start_date?: string | null
    end_date?: string | null
    use_types?: string[]
  }
  release_references: string[]
}

export interface BuildRequest {
  header: MessageHeader
  /** "3.8.2", "4.2" or "4.3" (required; there is no default) */
  version: string
  profile?: string | null
  /**
   * Releases (ERN 4.x: exactly one). When omitted or empty, the releases added with
   * `addRelease` / `addResource` are used.
   */
  releases?: Release[]
  deals?: Deal[]
  extensions?: Record<string, string> | null
  /** ERN 4.x (required): `NewReleaseMessage/@LanguageAndScriptCode`, e.g. `en` */
  language_and_script_code?: string | null
  /** ERN 4.3 (required): `NewReleaseMessage/@AvsVersionId` */
  avs_version_id?: string | null
}
