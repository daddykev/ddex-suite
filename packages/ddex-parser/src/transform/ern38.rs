//! ERN 3.8.x element extraction for the graph builder.
//!
//! `Release` and resource elements are read into an [`XNode`] tree (as for 4.x) and mapped
//! by following the 3.8.2 XSD: direct children, and the values the schema nests in
//! `ReleaseDetailsByTerritory` / `...DetailsByTerritory` blocks. Values from such a block
//! carry the block's territories (see [`TerritoryScope`]). Only values present in the XML
//! are recorded; nothing is synthesized.
//!
//! The repository's simplified 3.8 fixtures (`ReleaseTitle`, `DisplayArtist` directly under
//! `Release`) keep parsing as before.

use super::ern4::{
    copyrights, events, genres, parental_warnings, release_identifiers, resource_group, scope_of,
    XNode,
};
use ddex_core::models::common::{Identifier, IdentifierType, LocalizedString, TerritoryScope};
use ddex_core::models::graph::{
    Artist, DisplayText, LabelName, Release, ReleaseResourceReference, ReleaseType, Resource,
    ResourceType,
};
use ddex_core::models::versions::ERNVersion;

/// The `TerritoryCode` / `ExcludedTerritoryCode` list (and language) of a
/// `...DetailsByTerritory` block.
fn block_scope(block: &XNode) -> TerritoryScope {
    TerritoryScope {
        territory_codes: block
            .children_named("TerritoryCode")
            .filter_map(XNode::value)
            .collect(),
        excluded_territory_codes: block
            .children_named("ExcludedTerritoryCode")
            .filter_map(XNode::value)
            .collect(),
        is_default: None,
        language_and_script_code: block.attr("LanguageAndScriptCode").map(str::to_string),
    }
}

/// The node's `...DetailsByTerritory` blocks with their scopes, in document order.
fn detail_blocks<'a>(node: &'a XNode, name: &'a str) -> Vec<(&'a XNode, TerritoryScope)> {
    node.children_named(name)
        .map(|b| (b, block_scope(b)))
        .collect()
}

fn label_names(parent: &XNode, block: &TerritoryScope) -> Vec<LabelName> {
    parent
        .children_named("LabelName")
        .filter_map(|l| {
            Some(LabelName {
                name: l.value()?,
                label_name_type: l.attr("LabelNameType").map(str::to_string),
                scope: scope_of(l, Some(block)),
            })
        })
        .collect()
}

/// `TitleText` values of the `name` children of `parent`, in document order.
fn title_texts(parent: &XNode, name: &str) -> Vec<LocalizedString> {
    parent
        .children_named(name)
        .flat_map(|t| t.children_named("TitleText"))
        .filter_map(XNode::value)
        .map(LocalizedString::new)
        .collect()
}

fn release_type(value: String) -> ReleaseType {
    match value.as_str() {
        "Album" => ReleaseType::Album,
        "Single" => ReleaseType::Single,
        "EP" => ReleaseType::EP,
        "Compilation" => ReleaseType::Compilation,
        _ => ReleaseType::Other(value),
    }
}

/// Release `DisplayArtist`s as the 3.8.x path has always reported them: one artist per
/// `PartyName/FullName`, with the role `MainArtist`.
fn release_display_artists(parent: &XNode) -> impl Iterator<Item = Artist> + '_ {
    parent
        .children_named("DisplayArtist")
        .flat_map(|a| a.children_named("PartyName"))
        .filter_map(|pn| pn.child_value("FullName"))
        .map(|name| Artist {
            party_reference: None,
            artist_role: vec!["MainArtist".to_string()],
            display_artist_name: vec![LocalizedString::new(name)],
            sequence_number: None,
        })
}

/// Map a 3.8.x `Release`.
pub(crate) fn release(node: &XNode, version: ERNVersion) -> Release {
    let blocks = detail_blocks(node, "ReleaseDetailsByTerritory");

    // Titles: ReferenceTitle (or the simplified ReleaseTitle) and DisplayTitleText, in order.
    let mut release_title = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "ReferenceTitle" | "ReleaseTitle" => release_title.extend(
                child
                    .children_named("TitleText")
                    .filter_map(XNode::value)
                    .map(LocalizedString::new),
            ),
            "DisplayTitleText" => release_title.extend(child.value().map(LocalizedString::new)),
            _ => {}
        }
    }

    let mut display_artist: Vec<Artist> = release_display_artists(node).collect();
    let mut genre = Vec::new();
    let mut parental = Vec::new();
    let mut label_name = Vec::new();
    let mut p_line = copyrights(node, "P", None);
    let mut c_line = copyrights(node, "C", None);
    let mut release_date = events(node, "GlobalReleaseDate", None);
    let mut original_release_date = events(node, "GlobalOriginalReleaseDate", None);
    for (block, scope) in &blocks {
        display_artist.extend(release_display_artists(block));
        genre.extend(genres(block, Some(scope)));
        parental.extend(parental_warnings(block, Some(scope)));
        label_name.extend(label_names(block, scope));
        p_line.extend(copyrights(block, "P", Some(scope)));
        c_line.extend(copyrights(block, "C", Some(scope)));
        release_date.extend(events(block, "ReleaseDate", Some(scope)));
        original_release_date.extend(events(block, "OriginalReleaseDate", Some(scope)));
    }

    // ResourceGroup: the first one of the default block (Worldwide, else the first block
    // that has one).
    let group_block = TerritoryScope::pick_default(
        &blocks
            .iter()
            .filter(|(b, _)| b.child("ResourceGroup").is_some())
            .collect::<Vec<_>>(),
        |(_, s)| s,
    )
    .map(|(b, _)| *b);

    Release {
        release_reference: node
            .child_reference("ReleaseReference")
            .unwrap_or_else(|| format!("R_{:?}", version)),
        // Every direct ReleaseId (3.8.x allows several); never RelatedRelease's.
        release_id: node
            .children_named("ReleaseId")
            .flat_map(release_identifiers)
            .collect(),
        release_title,
        release_subtitle: None,
        // Release-level ReleaseType, else the first block's.
        release_type: node
            .child_value("ReleaseType")
            .or_else(|| {
                blocks
                    .iter()
                    .find_map(|(b, _)| b.child_value("ReleaseType"))
            })
            .map(release_type),
        genre,
        release_resource_reference_list: node
            .child("ReleaseResourceReferenceList")
            .map(|list| {
                list.children_named("ReleaseResourceReference")
                    .filter_map(XNode::reference)
                    .map(|r| ReleaseResourceReference {
                        resource_reference: r,
                        sequence_number: None,
                        disc_number: None,
                        track_number: None,
                        side: None,
                        is_hidden: false,
                        is_bonus: false,
                        extensions: None,
                        comments: None,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        display_artist,
        party_list: Vec::new(),
        release_date,
        territory_code: Vec::new(),
        excluded_territory_code: Vec::new(),
        display_title_text: Vec::new(),
        display_title: Vec::new(),
        display_artist_name: Vec::new(),
        resource_group: group_block
            .and_then(|b| b.child("ResourceGroup"))
            .map(resource_group),
        release_type_user_defined_value: None,
        parental_warning_type: parental.iter().map(|w| w.value.clone()).collect(),
        release_label_reference: Vec::new(),
        attributes: None,
        extensions: None,
        comments: None,
        parental_warnings: parental,
        p_line,
        c_line,
        original_release_date,
        label_name,
    }
}

/// `DisplayArtist` entries of a resource details block, with their roles and names.
fn resource_display_artists(block: &XNode) -> impl Iterator<Item = Artist> + '_ {
    block.children_named("DisplayArtist").map(|a| Artist {
        party_reference: None,
        artist_role: a
            .children_named("ArtistRole")
            .filter_map(XNode::value)
            .collect(),
        display_artist_name: a
            .children_named("PartyName")
            .filter_map(|pn| {
                pn.child_value("FullName").map(|full| LocalizedString {
                    text: full,
                    language_code: pn.attr("LanguageAndScriptCode").map(str::to_string),
                    script: None,
                })
            })
            .collect(),
        sequence_number: a.attr("SequenceNumber").and_then(|v| v.trim().parse().ok()),
    })
}

/// `DisplayArtistName` entries of a resource details block.
fn display_artist_names<'a>(
    block: &'a XNode,
    scope: &'a TerritoryScope,
) -> impl Iterator<Item = DisplayText> + 'a {
    block
        .children_named("DisplayArtistName")
        .filter_map(move |n| {
            Some(DisplayText {
                text: n.value()?,
                language_and_script_code: n
                    .attr("LanguageAndScriptCode")
                    .map(str::to_string)
                    .or_else(|| scope.language_and_script_code.clone()),
                applicable_territory_code: (scope.territory_codes.len() == 1)
                    .then(|| scope.territory_codes[0].clone()),
                is_default: None,
            })
        })
}

/// Map a 3.8.x resource (`SoundRecording`, `Video`, `Image`, `Text`, `SheetMusic`).
pub(crate) fn resource(node: &XNode, resource_type: ResourceType, version: ERNVersion) -> Resource {
    let details = format!("{}DetailsByTerritory", node.name);
    let blocks = detail_blocks(node, &details);

    // ISRC of SoundRecordingId / VideoId (or a simplified ResourceId).
    let resource_id = node
        .children
        .iter()
        .filter(|c| {
            matches!(
                c.name.as_str(),
                "SoundRecordingId" | "VideoId" | "ResourceId"
            )
        })
        .filter_map(|id| id.child_value("ISRC"))
        .map(|isrc| Identifier {
            id_type: IdentifierType::ISRC,
            namespace: None,
            value: isrc,
        })
        .collect();

    // Titles: ReferenceTitle (and a simplified DisplayTitleText), then each block's Title.
    let mut reference_title = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "ReferenceTitle" => reference_title.extend(
                child
                    .children_named("TitleText")
                    .filter_map(XNode::value)
                    .map(LocalizedString::new),
            ),
            "DisplayTitleText" => reference_title.extend(child.value().map(LocalizedString::new)),
            _ => {}
        }
    }

    let mut display_artist = Vec::new();
    let mut display_artist_name = Vec::new();
    let mut genre = Vec::new();
    let mut parental = Vec::new();
    let mut label_name = Vec::new();
    let mut p_line = Vec::new();
    let mut c_line = Vec::new();
    for (block, scope) in &blocks {
        reference_title.extend(title_texts(block, "Title"));
        display_artist.extend(resource_display_artists(block));
        display_artist_name.extend(display_artist_names(block, scope));
        genre.extend(genres(block, Some(scope)));
        parental.extend(parental_warnings(block, Some(scope)));
        label_name.extend(label_names(block, scope));
        p_line.extend(copyrights(block, "P", Some(scope)));
        c_line.extend(copyrights(block, "C", Some(scope)));
    }

    Resource {
        resource_reference: node
            .child_reference("ResourceReference")
            .unwrap_or_else(|| format!("RES_{:?}", version)),
        resource_type,
        resource_id,
        reference_title,
        duration: node
            .child_value("Duration")
            .and_then(|d| super::graph::parse_duration(d.trim()).ok()),
        technical_details: Vec::new(),
        rights_controller: Vec::new(),
        p_line,
        c_line,
        extensions: None,
        display_title_text: Vec::new(),
        display_title: Vec::new(),
        display_artist_name,
        display_artist,
        type_value: None,
        type_user_defined_value: None,
        parental_warning_type: parental.iter().map(|w| w.value.clone()).collect(),
        parental_warnings: parental,
        genre,
        label_name,
    }
}
