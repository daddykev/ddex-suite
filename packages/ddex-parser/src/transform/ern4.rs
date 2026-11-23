//! ERN 4.2 / 4.3 element extraction for the graph builder.
//!
//! Each top-level element of interest (`Release`, `TrackRelease`, `Party`, a resource) is
//! read into a small owned element tree ([`XNode`]) and then mapped to graph types by
//! following the XSD structure explicitly (direct children only, unless the schema nests a
//! value, e.g. 4.3 `SoundRecordingEdition/ResourceId/ISRC`). Only values present in the XML
//! are recorded; nothing is synthesized.
//!
//! The XML helpers and territorial-value mappers are shared with the 3.8.x reader (`ern38`).

use crate::error::ParseError;
use crate::parser::xml_validator::XmlValidator;
use ddex_core::models::common::{
    Copyright, Identifier, IdentifierType, LocalizedString, TerritoryScope,
};
use ddex_core::models::graph::{
    Artist, DisplayText, Genre, LinkedReleaseResourceReference, ParentalWarning, Party, Release,
    ReleaseEvent, ReleaseType, Resource, ResourceGroup, ResourceGroupContentItem, ResourceType,
    TrackRelease,
};
use ddex_core::models::versions::ERNVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::io::BufRead;

/// A minimal owned XML element: local name, attributes (local names), text and children.
#[derive(Debug, Clone, Default)]
pub(crate) struct XNode {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub text: String,
    pub children: Vec<XNode>,
}

impl XNode {
    fn from_start(e: &BytesStart) -> Result<Self, ParseError> {
        let mut attrs = Vec::new();
        for attr in e.attributes() {
            let attr =
                attr.map_err(|err| ParseError::XmlError(format!("Invalid attribute: {}", err)))?;
            let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).into_owned();
            let value = crate::xml_text::attr_value(&attr)
                .map_err(|err| ParseError::XmlError(format!("Invalid attribute value: {}", err)))?;
            attrs.push((key, value));
        }
        Ok(XNode {
            name: String::from_utf8_lossy(e.local_name().as_ref()).into_owned(),
            attrs,
            text: String::new(),
            children: Vec::new(),
        })
    }

    pub(crate) fn child(&self, name: &str) -> Option<&XNode> {
        self.children.iter().find(|c| c.name == name)
    }

    pub(crate) fn children_named<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a XNode> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub(crate) fn attr_bool(&self, name: &str) -> Option<bool> {
        match self.attr(name)?.trim() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }
    }

    /// The text of a leaf element exactly as written (`xs:string`-derived values keep their
    /// whitespace): `Some("")` for an empty element, `None` for an element with element
    /// children (whitespace between child elements is formatting, not text).
    pub(crate) fn value(&self) -> Option<String> {
        self.children.is_empty().then(|| self.text.clone())
    }

    /// The text of a reference (`xs:ID` / `xs:IDREF`, i.e. `xs:token`-derived) with the XSD
    /// `whiteSpace="collapse"` rule applied.
    pub(crate) fn reference(&self) -> Option<String> {
        self.value().map(|v| collapse(&v))
    }

    pub(crate) fn child_reference(&self, name: &str) -> Option<String> {
        self.child(name).and_then(XNode::reference)
    }

    pub(crate) fn child_value(&self, name: &str) -> Option<String> {
        self.child(name).and_then(XNode::value)
    }

    pub(crate) fn child_i32(&self, name: &str) -> Option<i32> {
        self.child(name).and_then(XNode::parsed)
    }

    /// The text parsed as a typed (`xs:integer`, `xs:gYear`, ...) value. These types collapse
    /// whitespace, so surrounding whitespace is ignored.
    pub(crate) fn parsed<T: std::str::FromStr>(&self) -> Option<T> {
        self.text.trim().parse().ok()
    }
}

/// XML end-of-line handling (XML 1.0 §2.11): `\r\n` and a lone `\r` become `\n`.
fn normalize_line_endings(text: &str) -> std::borrow::Cow<'_, str> {
    if text.contains('\r') {
        std::borrow::Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}

/// XSD `whiteSpace="collapse"`: tabs, line feeds and carriage returns become spaces, runs of
/// spaces become one, and leading/trailing spaces are removed.
pub(crate) fn collapse(text: &str) -> String {
    text.split([' ', '\t', '\n', '\r'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read the element whose start tag was just consumed into an [`XNode`], consuming events
/// up to and including its end tag. Every event is passed through the validator so its
/// stack stays consistent with the outer loop.
pub(crate) fn read_subtree<R: BufRead>(
    reader: &mut Reader<R>,
    validator: &mut XmlValidator,
    start: &BytesStart,
) -> Result<XNode, ParseError> {
    let mut stack = vec![XNode::from_start(start)?];
    let mut buf = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| ParseError::XmlError(format!("XML parsing error: {}", e)))?;
        validator.validate_event(&event, reader)?;
        if validator.get_depth() > 100 {
            return Err(ParseError::DepthLimitExceeded {
                depth: validator.get_depth(),
                limit: 100,
            });
        }
        match event {
            Event::Start(ref e) => stack.push(XNode::from_start(e)?),
            Event::Empty(ref e) => {
                let node = XNode::from_start(e)?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                }
            }
            Event::Text(ref e) => {
                if let Some(node) = stack.last_mut() {
                    // Line endings are normalized before entity expansion (XML 1.0 §2.11), so
                    // a `&#13;` reference still yields a carriage return.
                    let lossy = String::from_utf8_lossy(e.as_ref());
                    let raw = normalize_line_endings(&lossy);
                    node.text
                        .push_str(&quick_xml::escape::unescape(&raw).unwrap_or_default());
                }
            }
            // quick-xml >= 0.38 reports each reference (`&amp;`, `&#13;`) as its own event.
            Event::GeneralRef(ref e) => {
                if let Some(node) = stack.last_mut() {
                    node.text.push_str(&crate::xml_text::ref_text(e));
                }
            }
            Event::CData(ref e) => {
                if let Some(node) = stack.last_mut() {
                    node.text
                        .push_str(&normalize_line_endings(&String::from_utf8_lossy(
                            e.as_ref(),
                        )));
                }
            }
            Event::End(_) => {
                let node = stack
                    .pop()
                    .expect("stack is non-empty until the root closes");
                match stack.last_mut() {
                    Some(parent) => parent.children.push(node),
                    None => return Ok(node),
                }
            }
            Event::Eof => {
                return Err(ParseError::XmlError(
                    "Unexpected end of document inside element".to_string(),
                ))
            }
            _ => {}
        }
        buf.clear();
    }
}

/// Attributes of the 4.x root element (e.g. `LanguageAndScriptCode`, `AvsVersionId`),
/// keyed by their qualified names; namespace declarations are left out.
pub(crate) fn root_attributes(
    e: &BytesStart,
) -> Result<ddex_core::models::AttributeMap, ParseError> {
    let mut map = ddex_core::models::AttributeMap::new();
    for attr in e.attributes() {
        let attr =
            attr.map_err(|err| ParseError::XmlError(format!("Invalid attribute: {}", err)))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let value = crate::xml_text::attr_value(&attr)
            .map_err(|err| ParseError::XmlError(format!("Invalid attribute value: {}", err)))?;
        map.insert_str(&key, value);
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

fn display_text(node: &XNode, text: String) -> DisplayText {
    DisplayText {
        text,
        language_and_script_code: node.attr("LanguageAndScriptCode").map(str::to_string),
        applicable_territory_code: node.attr("ApplicableTerritoryCode").map(str::to_string),
        is_default: node.attr_bool("IsDefault"),
    }
}

/// Direct children named `name` whose own text is the value (`DisplayTitleText`,
/// `DisplayArtistName`).
fn display_texts(parent: &XNode, name: &str) -> Vec<DisplayText> {
    parent
        .children_named(name)
        .filter_map(|n| n.value().map(|v| display_text(n, v)))
        .collect()
}

/// `DisplayTitle/TitleText` entries; attributes come from `DisplayTitle`.
fn display_titles(parent: &XNode) -> Vec<DisplayText> {
    parent
        .children_named("DisplayTitle")
        .filter_map(|n| n.child_value("TitleText").map(|v| display_text(n, v)))
        .collect()
}

fn display_artists(parent: &XNode) -> Vec<Artist> {
    parent
        .children_named("DisplayArtist")
        .map(|n| Artist {
            party_reference: n.child_reference("ArtistPartyReference"),
            artist_role: n.child_value("DisplayArtistRole").into_iter().collect(),
            display_artist_name: Vec::new(),
            sequence_number: n.attr("SequenceNumber").and_then(|v| v.trim().parse().ok()),
        })
        .collect()
}

/// Identifiers in a `ReleaseId` element.
pub(crate) fn release_identifiers(release_id: &XNode) -> Vec<Identifier> {
    release_id
        .children
        .iter()
        .filter_map(|c| {
            let value = c.value()?;
            let (id_type, namespace) = match c.name.as_str() {
                "ICPN" => (IdentifierType::UPC, None),
                // Same mapping as the 3.8.x release parser.
                "GRIN" => (IdentifierType::GRid, None),
                "GRid" => (IdentifierType::GRID, None),
                "ISRC" => (IdentifierType::ISRC, None),
                "ProprietaryId" => (
                    IdentifierType::Proprietary,
                    c.attr("Namespace").map(str::to_string),
                ),
                _ => return None,
            };
            Some(Identifier {
                id_type,
                namespace,
                value,
            })
        })
        .collect()
}

fn linked_references(parent: &XNode) -> Vec<LinkedReleaseResourceReference> {
    parent
        .children_named("LinkedReleaseResourceReference")
        .filter_map(|n| {
            n.reference().map(|r| LinkedReleaseResourceReference {
                resource_reference: r,
                link_description: n.attr("LinkDescription").map(str::to_string),
            })
        })
        .collect()
}

pub(crate) fn resource_group(node: &XNode) -> ResourceGroup {
    ResourceGroup {
        resource_group_type: node.attr("ResourceGroupType").map(str::to_string),
        sequence_number: node.child_i32("SequenceNumber"),
        release_id: node
            .child("ReleaseId")
            .map(release_identifiers)
            .unwrap_or_default(),
        resource_group_release_reference: node.child_reference("ResourceGroupReleaseReference"),
        resource_groups: node
            .children_named("ResourceGroup")
            .map(resource_group)
            .collect(),
        content_items: node
            .children_named("ResourceGroupContentItem")
            .filter_map(|item| {
                // ReleaseResourceReference is mandatory; an item without one is skipped.
                Some(ResourceGroupContentItem {
                    sequence_number: item.child_i32("SequenceNumber"),
                    release_resource_reference: item.child_reference("ReleaseResourceReference")?,
                    linked_release_resource_references: linked_references(item),
                })
            })
            .collect(),
        linked_release_resource_references: linked_references(node),
    }
}

// ---------------------------------------------------------------------------
// Territorial values (shared with the 3.8.x reader)
// ---------------------------------------------------------------------------

/// The territory scope of `node`.
///
/// `block` is the scope of the 3.8.x `...DetailsByTerritory` block holding `node` (`None` in
/// 4.x, and for 3.8.x values written directly on a Release or resource). The node's own
/// `ApplicableTerritoryCode` (4.x) or `TerritoryCode` (3.8.x `EventDate`) attribute narrows
/// it to that territory; `IsDefault` and `LanguageAndScriptCode` come from the node, the
/// language falling back to the block's.
pub(crate) fn scope_of(node: &XNode, block: Option<&TerritoryScope>) -> TerritoryScope {
    let mut scope = block.cloned().unwrap_or_default();
    if let Some(code) = node
        .attr("ApplicableTerritoryCode")
        .or_else(|| node.attr("TerritoryCode"))
    {
        scope.territory_codes = vec![code.to_string()];
        scope.excluded_territory_codes.clear();
    }
    scope.is_default = node.attr_bool("IsDefault");
    if let Some(lang) = node.attr("LanguageAndScriptCode") {
        scope.language_and_script_code = Some(lang.to_string());
    }
    scope
}

/// `Genre` entries (`GenreText` is mandatory; a `Genre` without one is skipped).
pub(crate) fn genres(parent: &XNode, block: Option<&TerritoryScope>) -> Vec<Genre> {
    parent
        .children_named("Genre")
        .filter_map(|g| {
            Some(Genre {
                genre_text: g.child_value("GenreText")?,
                sub_genre: g.child_value("SubGenre"),
                attributes: None,
                extensions: None,
                comments: None,
                scope: scope_of(g, block),
            })
        })
        .collect()
}

/// `PLine` (`kind` = "P") or `CLine` (`kind` = "C") entries. `PLineText`/`CLineText` is
/// mandatory; an entry without it is skipped. `Year` is an `xs:gYear`: a value that is not a
/// plain year stays `None`.
pub(crate) fn copyrights(
    parent: &XNode,
    kind: &str,
    block: Option<&TerritoryScope>,
) -> Vec<Copyright> {
    let element = format!("{kind}Line");
    let text = format!("{kind}LineText");
    let company = format!("{kind}LineCompany");
    parent
        .children_named(&element)
        .filter_map(|line| {
            Some(Copyright {
                text: line.child_value(&text)?,
                year: line.child("Year").and_then(XNode::parsed),
                owner: line.child_value(&company),
                scope: scope_of(line, block),
            })
        })
        .collect()
}

/// `ParentalWarningType` entries with their attributes.
pub(crate) fn parental_warnings(
    parent: &XNode,
    block: Option<&TerritoryScope>,
) -> Vec<ParentalWarning> {
    parent
        .children_named("ParentalWarningType")
        .filter_map(|w| {
            Some(ParentalWarning {
                value: w.value()?,
                namespace: w.attr("Namespace").map(str::to_string),
                user_defined_value: w.attr("UserDefinedValue").map(str::to_string),
                scope: scope_of(w, block),
            })
        })
        .collect()
}

/// Date elements named `name` (`ddex_IsoDate`: `YYYY`, `YYYY-MM` or `YYYY-MM-DD`). The text
/// is kept as written; `event_date` is set only for a full date (midnight UTC).
pub(crate) fn events(
    parent: &XNode,
    name: &str,
    block: Option<&TerritoryScope>,
) -> Vec<ReleaseEvent> {
    parent
        .children_named(name)
        .filter_map(|d| {
            let date = d.value()?;
            let scope = scope_of(d, block);
            Some(ReleaseEvent {
                release_event_type: name.to_string(),
                event_date: full_date(&date),
                territory: (scope.territory_codes.len() == 1)
                    .then(|| scope.territory_codes[0].clone()),
                date: Some(date),
                is_approximate: d.attr_bool("IsApproximate"),
                scope,
                extensions: None,
                comments: None,
            })
        })
        .collect()
}

/// A full `YYYY-MM-DD` date (surrounding whitespace collapsed, as for any `xs:` typed value)
/// at midnight UTC; `None` for partial or invalid dates.
pub(crate) fn full_date(text: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc())
}

/// Text values of the direct children named `name`, in document order.
fn values(parent: &XNode, name: &str) -> Vec<String> {
    parent
        .children_named(name)
        .filter_map(XNode::value)
        .collect()
}

/// Reference values (collapsed) of the direct children named `name`, in document order.
fn references(parent: &XNode, name: &str) -> Vec<String> {
    parent
        .children_named(name)
        .filter_map(XNode::reference)
        .collect()
}

fn to_localized(texts: &[DisplayText]) -> Vec<LocalizedString> {
    texts
        .iter()
        .map(|t| LocalizedString::new(t.text.clone()))
        .collect()
}

// ---------------------------------------------------------------------------
// Element mappers
// ---------------------------------------------------------------------------

/// Map a 4.x `Release`.
pub(crate) fn release(node: &XNode, version: ERNVersion) -> Release {
    let display_title_text = display_texts(node, "DisplayTitleText");
    Release {
        // Same fallback as the 3.8.x path for the (schema-mandatory) reference.
        release_reference: node
            .child_reference("ReleaseReference")
            .unwrap_or_else(|| format!("R_{:?}", version)),
        // Only the Release's own ReleaseId: never RelatedRelease or ResourceGroup ReleaseIds.
        release_id: node
            .child("ReleaseId")
            .map(release_identifiers)
            .unwrap_or_default(),
        release_title: to_localized(&display_title_text),
        release_subtitle: None,
        release_type: node.child_value("ReleaseType").map(|t| match t.as_str() {
            "Album" => ReleaseType::Album,
            "Single" => ReleaseType::Single,
            "EP" => ReleaseType::EP,
            "Compilation" => ReleaseType::Compilation,
            _ => ReleaseType::Other(t),
        }),
        genre: genres(node, None),
        release_resource_reference_list: Vec::new(),
        display_artist: display_artists(node),
        party_list: Vec::new(),
        release_date: events(node, "ReleaseDate", None),
        territory_code: Vec::new(),
        excluded_territory_code: Vec::new(),
        display_title_text,
        display_title: display_titles(node),
        display_artist_name: display_texts(node, "DisplayArtistName"),
        resource_group: node.child("ResourceGroup").map(resource_group),
        release_type_user_defined_value: node
            .child("ReleaseType")
            .and_then(|t| t.attr("UserDefinedValue"))
            .map(str::to_string),
        parental_warning_type: values(node, "ParentalWarningType"),
        release_label_reference: references(node, "ReleaseLabelReference"),
        attributes: None,
        extensions: None,
        comments: None,
        c_line: copyrights(node, "C", None),
        label_name: Vec::new(),
        original_release_date: events(node, "OriginalReleaseDate", None),
        p_line: copyrights(node, "P", None),
        parental_warnings: parental_warnings(node, None),
    }
}

/// Map a 4.x `TrackRelease`. Returns `None` when the mandatory `ReleaseReference` or
/// `ReleaseResourceReference` is missing (no reference is invented).
pub(crate) fn track_release(node: &XNode) -> Option<TrackRelease> {
    Some(TrackRelease {
        release_reference: node.child_reference("ReleaseReference")?,
        release_id: node
            .child("ReleaseId")
            .map(release_identifiers)
            .unwrap_or_default(),
        display_title_text: display_texts(node, "DisplayTitleText"),
        display_title: display_titles(node),
        release_resource_reference: node.child_reference("ReleaseResourceReference")?,
        linked_release_resource_references: linked_references(node),
        release_label_reference: references(node, "ReleaseLabelReference"),
        genre: genres(node, None),
        is_main_release: node.attr_bool("IsMainRelease"),
    })
}

/// Map a 4.x `PartyList/Party`.
pub(crate) fn party(node: &XNode) -> Party {
    let mut party_id = Vec::new();
    let mut isni = None;
    let mut ipi = None;
    for pid in node.children_named("PartyId") {
        if let Some(v) = pid.value() {
            party_id.push(Identifier {
                id_type: IdentifierType::Proprietary,
                namespace: pid.attr("Namespace").map(str::to_string),
                value: v,
            });
        }
        for c in &pid.children {
            let Some(value) = c.value() else { continue };
            let (id_type, namespace) = match c.name.as_str() {
                "ISNI" => {
                    isni.get_or_insert_with(|| value.clone());
                    (IdentifierType::ISNI, None)
                }
                "IpiNameNumber" => {
                    ipi.get_or_insert_with(|| value.clone());
                    (IdentifierType::IPI, None)
                }
                "ProprietaryId" => (
                    IdentifierType::Proprietary,
                    c.attr("Namespace").map(str::to_string),
                ),
                "DPID" => (IdentifierType::Proprietary, Some("DPID".to_string())),
                _ => continue,
            };
            party_id.push(Identifier {
                id_type,
                namespace,
                value,
            });
        }
    }
    Party {
        party_reference: node.child_reference("PartyReference"),
        party_id,
        isni,
        ipi,
        party_name: node
            .children_named("PartyName")
            .filter_map(|pn| {
                pn.child_value("FullName").map(|full| LocalizedString {
                    text: full,
                    language_code: pn.attr("LanguageAndScriptCode").map(str::to_string),
                    script: None,
                })
            })
            .collect(),
        party_role: Vec::new(),
        contact_details: None,
    }
}

/// Map a 4.x resource (`SoundRecording`, `Video`, `Image`, `Text`, `SheetMusic`).
pub(crate) fn resource(node: &XNode, resource_type: ResourceType, version: ERNVersion) -> Resource {
    // ISRC: 4.2 `ResourceId/ISRC`; 4.3 `SoundRecordingEdition|VideoEdition/ResourceId/ISRC`.
    let mut resource_id = Vec::new();
    let editions = node
        .children
        .iter()
        .filter(|c| c.name.ends_with("Edition"))
        .flat_map(|e| e.children_named("ResourceId"));
    for rid in node.children_named("ResourceId").chain(editions) {
        if let Some(isrc) = rid.child_value("ISRC") {
            resource_id.push(Identifier {
                id_type: IdentifierType::ISRC,
                namespace: None,
                value: isrc,
            });
        }
        for pid in rid.children_named("ProprietaryId") {
            if let Some(value) = pid.value() {
                resource_id.push(Identifier {
                    id_type: IdentifierType::Proprietary,
                    namespace: pid.attr("Namespace").map(str::to_string),
                    value,
                });
            }
        }
    }
    let display_title_text = display_texts(node, "DisplayTitleText");
    // P/C lines: on the resource (4.2, and 4.3 Image/Text/SheetMusic), or in the 4.3
    // SoundRecordingEdition / VideoEdition.
    let mut p_line = copyrights(node, "P", None);
    let mut c_line = copyrights(node, "C", None);
    for edition in node.children.iter().filter(|c| c.name.ends_with("Edition")) {
        p_line.extend(copyrights(edition, "P", None));
        c_line.extend(copyrights(edition, "C", None));
    }
    Resource {
        // Same fallback as the 3.8.x path for the (schema-mandatory) reference.
        resource_reference: node
            .child_reference("ResourceReference")
            .unwrap_or_else(|| format!("RES_{:?}", version)),
        resource_type,
        resource_id,
        reference_title: to_localized(&display_title_text),
        duration: node
            .child_value("Duration")
            .and_then(|d| super::graph::parse_duration(d.trim()).ok()),
        technical_details: Vec::new(),
        rights_controller: Vec::new(),
        p_line,
        c_line,
        extensions: None,
        display_title_text,
        display_title: display_titles(node),
        display_artist_name: display_texts(node, "DisplayArtistName"),
        display_artist: display_artists(node),
        type_value: node.child_value("Type"),
        type_user_defined_value: node
            .child("Type")
            .and_then(|t| t.attr("UserDefinedValue"))
            .map(str::to_string),
        parental_warning_type: values(node, "ParentalWarningType"),
        genre: Vec::new(),
        label_name: Vec::new(),
        parental_warnings: parental_warnings(node, None),
    }
}
