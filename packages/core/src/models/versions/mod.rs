// core/src/models/versions/mod.rs
//! Version-specific model variations for ERN standards

use serde::{Deserialize, Serialize};

pub mod version;
pub use version::ERNVersion;

pub mod ern_382;
pub mod ern_42;
pub mod ern_43;
pub mod common;

/// Version-specific differences in DDEX ERN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDifferences {
    pub version: ERNVersion,
    pub namespace_uri: String,
    pub schema_location: String,
    pub features: VersionFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFeatures {
    // Features introduced or changed in each version
    pub supports_message_audit_trail: bool,
    pub supports_release_profile: bool,
    pub supports_technical_instantiation: bool,
    pub supports_deal_reference: bool,
    pub supports_resource_group: bool,
    pub supports_chapter_information: bool,
    pub deal_terms_structure: DealTermsVersion,
    pub party_descriptor_type: PartyDescriptorVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DealTermsVersion {
    Legacy,     // 3.8.2 style
    Standard,   // 4.2 style
    Extended,   // 4.3 style with additional fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartyDescriptorVersion {
    Basic,      // 3.8.2
    Enhanced,   // 4.2+
}

impl VersionDifferences {
    pub fn for_version(version: ERNVersion) -> Self {
        match version {
            ERNVersion::V3_8_2 => Self {
                version,
                namespace_uri: "http://ddex.net/xml/ern/382".to_string(),
                schema_location: "http://ddex.net/xml/ern/382/release-notification.xsd".to_string(),
                features: VersionFeatures {
                    supports_message_audit_trail: false,
                    supports_release_profile: false,
                    supports_technical_instantiation: false,
                    supports_deal_reference: false,
                    supports_resource_group: false,
                    supports_chapter_information: false,
                    deal_terms_structure: DealTermsVersion::Legacy,
                    party_descriptor_type: PartyDescriptorVersion::Basic,
                },
            },
            ERNVersion::V4_2 => Self {
                version,
                namespace_uri: "http://ddex.net/xml/ern/42".to_string(),
                schema_location: "http://ddex.net/xml/ern/42/release-notification.xsd".to_string(),
                features: VersionFeatures {
                    supports_message_audit_trail: true,
                    supports_release_profile: true,
                    supports_technical_instantiation: true,
                    supports_deal_reference: true,
                    supports_resource_group: false,
                    supports_chapter_information: false,
                    deal_terms_structure: DealTermsVersion::Standard,
                    party_descriptor_type: PartyDescriptorVersion::Enhanced,
                },
            },
            ERNVersion::V4_3 => Self {
                version,
                namespace_uri: "http://ddex.net/xml/ern/43".to_string(),
                schema_location: "http://ddex.net/xml/ern/43/release-notification.xsd".to_string(),
                features: VersionFeatures {
                    supports_message_audit_trail: true,
                    supports_release_profile: true,
                    supports_technical_instantiation: true,
                    supports_deal_reference: true,
                    supports_resource_group: true,
                    supports_chapter_information: true,
                    deal_terms_structure: DealTermsVersion::Extended,
                    party_descriptor_type: PartyDescriptorVersion::Enhanced,
                },
            },
        }
    }
}