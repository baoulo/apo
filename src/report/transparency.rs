//! Transparency notes for disabled / skipped / overridden analysis inputs.

use serde::{Deserialize, Serialize};

pub use crate::packs::SkippedToolingNote;

/// Explicit analysis adjustments that would otherwise be invisible in scores.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transparency {
    /// Hygiene rule ids excluded via `[analyze].rule_disable` / config.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled_rules: Vec<String>,
    /// Built-in pack ids replaced by an external pack with the same id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overridden_packs: Vec<String>,
    /// Tooling needles skipped (unknown `maps_to`, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_tooling: Vec<SkippedToolingNote>,
    /// Rule ids observed as `NotApplicable` (still listed under findings).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_applicable_rules: Vec<String>,
}

impl Transparency {
    pub fn is_empty(&self) -> bool {
        self.disabled_rules.is_empty()
            && self.overridden_packs.is_empty()
            && self.skipped_tooling.is_empty()
            && self.not_applicable_rules.is_empty()
    }

    pub fn sort(&mut self) {
        self.disabled_rules.sort();
        self.disabled_rules.dedup();
        self.overridden_packs.sort();
        self.overridden_packs.dedup();
        self.not_applicable_rules.sort();
        self.not_applicable_rules.dedup();
        self.skipped_tooling
            .sort_by(|a, b| (&a.source, &a.id).cmp(&(&b.source, &b.id)));
    }
}
