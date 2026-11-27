"""Types for the build request accepted by ``DdexBuilder.build(request)``,
``DdexBuilder.build_with_statistics(request=...)``, ``DdexBuilder.from_dataframe(df, request=...)``
and ``batch_build([...])``.

The request mirrors the Rust core ``BuildRequest`` (same snake_case field names) and is passed
to the core builder unchanged. Nothing is filled in: when the target ERN version's schema
requires a value the request does not carry, the build fails with an error naming the field,
e.g. ``ERN 4.3 requires Release/ReleaseType (releases[0].release_type)``.

Fields marked "ERN 4.x" are only written for 4.2/4.3 targets.
"""

from typing import Dict, List, Optional, TypedDict, Union


class AvsValueDict(TypedDict, total=False):
    """A DDEX allowed-value-set term with ``UserDefined`` details."""

    value: str
    namespace: Optional[str]
    user_defined_value: Optional[str]


#: A DDEX allowed-value-set term (``"Album"``), or a dict for ``UserDefined`` values.
AvsValue = Union[str, AvsValueDict]


class _LocalizedStringRequired(TypedDict):
    text: str


class LocalizedString(_LocalizedStringRequired, total=False):
    language_code: Optional[str]


class _PartyRequired(TypedDict):
    party_name: List[LocalizedString]


class Party(_PartyRequired, total=False):
    #: ERN 4.x: required for sender/recipient, a DDEX Party ID (``PADPIDA...``)
    party_id: Optional[str]
    party_reference: Optional[str]


class _MessageHeaderRequired(TypedDict):
    #: Required: the builder never generates it
    message_id: str
    message_sender: Party
    message_recipient: Party
    #: Required xs:dateTime, e.g. ``2024-01-01T00:00:00Z`` (the builder reads no clock)
    message_created_date_time: str


class MessageHeader(_MessageHeaderRequired, total=False):
    #: ``LiveMessage`` or ``TestMessage``
    message_control_type: Optional[str]


class ProprietaryId(TypedDict):
    namespace: str
    value: str


class ReleaseIds(TypedDict, total=False):
    """ERN 4.x: all identifiers of a release, written as one ``ReleaseId``."""

    grid: Optional[str]
    icpn: Optional[str]
    proprietary_ids: List[ProprietaryId]


class _ContentItemRequired(TypedDict):
    resource_reference: str


class ResourceGroupContentItem(_ContentItemRequired, total=False):
    sequence_number: Optional[int]


class ResourceGroup(TypedDict, total=False):
    """ERN 4.x: a ``ResourceGroup`` node (nested groups need ``resource_group_type``)."""

    resource_group_type: Optional[str]
    sequence_number: Optional[int]
    resource_groups: List["ResourceGroup"]
    content_items: List[ResourceGroupContentItem]


class _TrackReleaseRequired(TypedDict):
    release_reference: str
    release_ids: ReleaseIds


class TrackRelease(_TrackReleaseRequired, total=False):
    """ERN 4.x: a ``TrackRelease`` for one track (``genre`` and ``label`` are required)."""

    title: Optional[str]
    genre: Optional[str]
    sub_genre: Optional[str]
    label: Optional[str]


class _TrackRequired(TypedDict):
    track_id: str
    title: str
    #: ISO 8601 duration, e.g. ``PT3M45S``
    duration: str


class Track(_TrackRequired, total=False):
    #: Left out (or None): no ISRC; the track then needs ``proprietary_ids``
    isrc: Optional[str]
    #: ERN 4.x: required; ERN 3.8.x: left out (or None) writes no display artist
    artist: Optional[str]
    resource_reference: Optional[str]
    #: ERN 4.x: ``"SoundRecording"`` (default) or ``"Video"``
    resource_kind: str
    #: ERN 4.x (required): the resource's DDEX ``Type``, e.g. ``MusicalWorkSoundRecording``
    resource_type: Optional[AvsValue]
    #: ERN 4.x (required): ``ParentalWarningType``, e.g. ``NotExplicit``
    parental_warning_type: Optional[str]
    #: ERN 4.x: ``DisplayArtistRole`` of ``artist`` (default ``MainArtist``)
    artist_role: Optional[AvsValue]
    disc_number: Optional[int]
    sequence_number: Optional[int]
    proprietary_ids: List[ProprietaryId]
    track_release: Optional[TrackRelease]


class _ReleaseRequired(TypedDict):
    title: List[LocalizedString]
    tracks: List[Track]


class Release(_ReleaseRequired, total=False):
    #: Written as ``ReleaseId/GRid`` unless ``release_ids`` is set (ERN 4.x); left out (or
    #: None): no GRid
    release_id: Optional[str]
    #: Main artist (ERN 4.x: required, ``DisplayArtistName`` and a ``MainArtist``
    #: ``DisplayArtist``; ERN 3.8.x: optional)
    artist: Optional[str]
    release_reference: Optional[str]
    #: ERN 4.x (required): label name, referenced by ``ReleaseLabelReference``
    label: Optional[str]
    release_date: Optional[str]
    upc: Optional[str]
    resource_references: Optional[List[str]]
    #: ERN 4.x (required), e.g. ``Album``
    release_type: Optional[AvsValue]
    #: ERN 4.x (required)
    genre: Optional[str]
    sub_genre: Optional[str]
    #: ERN 4.x (required), e.g. ``NotExplicit``
    parental_warning_type: Optional[str]
    artist_role: Optional[AvsValue]
    release_ids: Optional[ReleaseIds]
    resource_group: Optional[ResourceGroup]


class _DealTermsRequired(TypedDict):
    territory_code: List[str]


class DealTerms(_DealTermsRequired, total=False):
    #: Left out (or None): no ``CommercialModelType``
    commercial_model_type: Optional[str]
    #: YYYY-MM-DD (ERN 4.x: required)
    start_date: Optional[str]
    end_date: Optional[str]
    use_types: List[str]


class _DealRequired(TypedDict):
    deal_terms: DealTerms
    release_references: List[str]


class Deal(_DealRequired, total=False):
    deal_reference: Optional[str]


class _BuildRequestRequired(TypedDict):
    header: MessageHeader
    #: ``"3.8.2"``, ``"4.2"`` or ``"4.3"`` (required; there is no default)
    version: str


class BuildRequest(_BuildRequestRequired, total=False):
    profile: Optional[str]
    #: Releases (ERN 4.x: exactly one). When omitted or empty, the releases added with
    #: ``add_release`` / ``add_resource`` (or the DataFrame's rows) are used.
    releases: List[Release]
    deals: List[Deal]
    extensions: Optional[Dict[str, str]]
    #: ERN 4.x (required): ``NewReleaseMessage/@LanguageAndScriptCode``, e.g. ``en``
    language_and_script_code: Optional[str]
    #: ERN 4.3 (required): ``NewReleaseMessage/@AvsVersionId``
    avs_version_id: Optional[str]


__all__ = [
    "AvsValue",
    "AvsValueDict",
    "BuildRequest",
    "Deal",
    "DealTerms",
    "LocalizedString",
    "MessageHeader",
    "Party",
    "ProprietaryId",
    "Release",
    "ReleaseIds",
    "ResourceGroup",
    "ResourceGroupContentItem",
    "Track",
    "TrackRelease",
]


class RequestFromXml(TypedDict):
    """The result of ``request_from_xml``."""

    #: The build request, in the shape ``DdexBuilder.build`` accepts (``None`` where the
    #: parsed message has no value)
    request: BuildRequest
    #: The dotted path of every non-empty parsed value the request cannot hold (e.g.
    #: ``releases[0].p_line``)
    unmapped: List[str]
