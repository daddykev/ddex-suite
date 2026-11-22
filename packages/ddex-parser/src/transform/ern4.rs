//! ERN 4.2 / 4.3 element extraction for the graph builder.
//!
//! Each top-level element of interest (`Release`, `TrackRelease`, `Party`, a resource) is
//! read into a small owned element tree ([`XNode`]) and then mapped to graph types by
//! following the XSD structure explicitly (direct children only, unless the schema nests a
//! value, e.g. 4.3 `SoundRecordingEdition/ResourceId/ISRC`). Only values present in the XML
//! are recorded; nothing is synthesized.
//!
//! ERN 3.8.x documents never reach this module.

use crate::error::ParseError;
use crate::parser::xml_validator::XmlValidator;
use ddex_core::models::common::{Identifier, IdentifierType, LocalizedString};
use ddex_core::models::graph::{
    Artist, DisplayText, Genre, LinkedReleaseResourceReference, Party, Release, ReleaseType,
    Resource, ResourceGroup, ResourceGroupContentItem, ResourceType, TrackRelease,
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
            let value = attr
                .unescape_value()
                .map_err(|err| ParseError::XmlError(format!("Invalid attribute value: {}", err)))?
                .into_owned();
            attrs.push((key, value));
        }
        Ok(XNode {
            name: String::from_utf8_lossy(e.local_name().as_ref()).into_owned(),
            attrs,
            text: String::new(),
            children: Vec::new(),
        })
    }

    fn child(&self, name: &str) -> Option<&XNode> {
        self.children.iter().find(|c| c.name == name)
    }

    fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a XNode> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn attr_bool(&self, name: &str) -> Option<bool> {
        match self.attr(name)?.trim() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }
    }

    /// Trimmed text content, `None` when empty.
    fn value(&self) -> Option<String> {
        let t = self.text.trim();
        (!t.is_empty()).then(|| t.to_string())
    }

    fn child_value(&self, name: &str) -> Option<String> {
        self.child(name).and_then(XNode::value)
    }

    fn child_i32(&self, name: &str) -> Option<i32> {
        self.child_value(name).and_then(|v| v.parse().ok())
    }
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
                    node.text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Event::CData(ref e) => {
                if let Some(node) = stack.last_mut() {
                    node.text.push_str(&String::from_utf8_lossy(e.as_ref()));
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
        let attr = attr.map_err(|err| ParseError::XmlError(format!("Invalid attribute: {}", err)))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let value = attr
            .unescape_value()
            .map_err(|err| ParseError::XmlError(format!("Invalid attribute value: {}", err)))?
            .into_owned();
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
            party_reference: n.child_value("ArtistPartyReference"),
            artist_role: n.child_value("DisplayArtistRole").into_iter().collect(),
            display_artist_name: Vec::new(),
            sequence_number: n.attr("SequenceNumber").and_then(|v| v.trim().parse().ok()),
        })
        .collect()
}

/// Identifiers in a `ReleaseId` element.
fn release_identifiers(release_id: &XNode) -> Vec<Identifier> {
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
            n.value().map(|r| LinkedReleaseResourceReference {
                resource_reference: r,
                link_description: n.attr("LinkDescription").map(str::to_string),
            })
        })
        .collect()
}

fn resource_group(node: &XNode) -> ResourceGroup {
    ResourceGroup {
        resource_group_type: node.attr("ResourceGroupType").map(str::to_string),
        sequence_number: node.child_i32("SequenceNumber"),
        release_id: node
            .child("ReleaseId")
            .map(release_identifiers)
            .unwrap_or_default(),
        resource_group_release_reference: node.child_value("ResourceGroupReleaseReference"),
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
                    release_resource_reference: item.child_value("ReleaseResourceReference")?,
                    linked_release_resource_references: linked_references(item),
                })
            })
            .collect(),
        linked_release_resource_references: linked_references(node),
    }
}

/// `Genre` entries (`GenreText` is mandatory; a `Genre` without one is skipped).
fn genres(parent: &XNode) -> Vec<Genre> {
    parent
        .children_named("Genre")
        .filter_map(|g| {
            Some(Genre {
                genre_text: g.child_value("GenreText")?,
                sub_genre: g.child_value("SubGenre"),
                attributes: None,
                extensions: None,
                comments: None,
            })
        })
        .collect()
}

/// Text values of the direct children named `name`, in document order.
fn values(parent: &XNode, name: &str) -> Vec<String> {
    parent.children_named(name).filter_map(XNode::value).collect()
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
            .child_value("ReleaseReference")
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
        genre: genres(node),
        release_resource_reference_list: Vec::new(),
        display_artist: display_artists(node),
        party_list: Vec::new(),
        release_date: Vec::new(),
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
        release_label_reference: values(node, "ReleaseLabelReference"),
        attributes: None,
        extensions: None,
        comments: None,
    }
}

/// Map a 4.x `TrackRelease`. Returns `None` when the mandatory `ReleaseReference` or
/// `ReleaseResourceReference` is missing (no reference is invented).
pub(crate) fn track_release(node: &XNode) -> Option<TrackRelease> {
    Some(TrackRelease {
        release_reference: node.child_value("ReleaseReference")?,
        release_id: node
            .child("ReleaseId")
            .map(release_identifiers)
            .unwrap_or_default(),
        display_title_text: display_texts(node, "DisplayTitleText"),
        display_title: display_titles(node),
        release_resource_reference: node.child_value("ReleaseResourceReference")?,
        linked_release_resource_references: linked_references(node),
        release_label_reference: values(node, "ReleaseLabelReference"),
        genre: genres(node),
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
        party_reference: node.child_value("PartyReference"),
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
    Resource {
        // Same fallback as the 3.8.x path for the (schema-mandatory) reference.
        resource_reference: node
            .child_value("ResourceReference")
            .unwrap_or_else(|| format!("RES_{:?}", version)),
        resource_type,
        resource_id,
        reference_title: to_localized(&display_title_text),
        duration: node
            .child_value("Duration")
            .and_then(|d| super::graph::parse_duration(&d).ok()),
        technical_details: Vec::new(),
        rights_controller: Vec::new(),
        p_line: Vec::new(),
        c_line: Vec::new(),
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
    }
}
