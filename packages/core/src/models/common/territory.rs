// core/src/models/common/territory.rs
//! Territory and copyright types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerritoryCode {
    pub code: String,
    pub excluded: bool,
}

/// A P line or C line (`PLine` / `CLine`): `Year`, `PLineText`/`CLineText` and, as
/// `owner`, `PLineCompany`/`CLineCompany`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Copyright {
    pub text: String,
    pub year: Option<i32>,
    pub owner: Option<String>,
    /// Where this line applies (see [`TerritoryScope`]).
    #[serde(default)]
    pub scope: TerritoryScope,
}

/// Where a territorial value applies.
///
/// - ERN 4.x: the value's own `ApplicableTerritoryCode`, `IsDefault` and
///   `LanguageAndScriptCode` attributes.
/// - ERN 3.8.x: the `TerritoryCode` / `ExcludedTerritoryCode` list of the
///   `...DetailsByTerritory` block the value was read from (empty for a value written directly
///   on the `Release` or resource), plus the value's own `LanguageAndScriptCode`.
///
/// Only what the XML says is recorded: an absent attribute stays `None` / empty.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TerritoryScope {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub territory_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_territory_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_and_script_code: Option<String>,
}

impl TerritoryScope {
    /// True when the value applies everywhere: no territory restriction, or `Worldwide`.
    pub fn is_worldwide(&self) -> bool {
        self.excluded_territory_codes.is_empty()
            && (self.territory_codes.is_empty()
                || self.territory_codes.iter().any(|c| c == "Worldwide"))
    }

    /// The default entry of a territorial list: the first with `IsDefault="true"`, else the
    /// first that applies worldwide (see [`TerritoryScope::is_worldwide`]), else the first.
    pub fn pick_default<T>(items: &[T], scope: impl Fn(&T) -> &TerritoryScope) -> Option<&T> {
        items
            .iter()
            .find(|i| scope(i).is_default == Some(true))
            .or_else(|| items.iter().find(|i| scope(i).is_worldwide()))
            .or_else(|| items.first())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub territory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidityPeriod {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}
