//! SARIF 2.1.0 export for unified evidence packs (GitHub Code Scanning).

use serde_json::{Value, json};
use std::path::Path;

use crate::error::Result;
use crate::evidence::{Finding, Status};
use crate::report::pack::EvidencePack;

/// Map observational status to SARIF level.
fn level_for(status: Status) -> &'static str {
    match status {
        Status::Missing => "error",
        Status::Partial | Status::Unknown => "warning",
        Status::Present | Status::Enforced | Status::NotApplicable => "note",
    }
}

fn result_from_finding(prefix: &str, f: &Finding) -> Option<Value> {
    if matches!(
        f.status,
        Status::NotApplicable | Status::Enforced | Status::Present
    ) && !f.status.is_gap()
    {
        // Still emit Present as note so scanners see coverage; skip NA to reduce noise.
        if matches!(f.status, Status::NotApplicable) {
            return None;
        }
    }
    if matches!(f.status, Status::NotApplicable) {
        return None;
    }

    let rule_id = format!("{prefix}{}", f.rule);
    let mut locations = Vec::new();
    for item in &f.evidence {
        if let Some(path) = &item.path {
            let mut loc = json!({
                "physicalLocation": {
                    "artifactLocation": { "uri": path }
                }
            });
            if let Some(line) = item.line {
                loc["physicalLocation"]["region"] = json!({ "startLine": line });
            }
            locations.push(loc);
        }
    }
    if locations.is_empty() {
        locations.push(json!({
            "physicalLocation": {
                "artifactLocation": { "uri": "." }
            }
        }));
    }

    Some(json!({
        "ruleId": rule_id,
        "level": level_for(f.status),
        "message": { "text": f.summary },
        "locations": locations
    }))
}

fn rule_descriptor(prefix: &str, f: &Finding) -> Value {
    json!({
        "id": format!("{prefix}{}", f.rule),
        "name": f.rule,
        "shortDescription": { "text": f.summary },
        "fullDescription": {
            "text": f.remediation.clone().unwrap_or_else(|| f.summary.clone())
        },
        "defaultConfiguration": {
            "level": level_for(f.status)
        }
    })
}

/// Build SARIF 2.1.0 document for a pack.
pub fn pack_to_sarif(pack: &EvidencePack) -> Value {
    let mut rules = Vec::new();
    let mut results = Vec::new();
    let mut seen_rules = std::collections::BTreeSet::new();

    for f in &pack.hygiene.findings {
        let prefix = "hygiene.";
        if let Some(r) = result_from_finding(prefix, f) {
            results.push(r);
        }
        let id = format!("{prefix}{}", f.rule);
        if seen_rules.insert(id) {
            rules.push(rule_descriptor(prefix, f));
        }
    }
    for f in &pack.evidence.knowledge.findings {
        let prefix = "knowledge.";
        if let Some(r) = result_from_finding(prefix, f) {
            results.push(r);
        }
        let id = format!("{prefix}{}", f.rule);
        if seen_rules.insert(id) {
            rules.push(rule_descriptor(prefix, f));
        }
    }
    for f in &pack.evidence.ai.findings {
        let prefix = "ai.";
        if let Some(r) = result_from_finding(prefix, f) {
            results.push(r);
        }
        let id = format!("{prefix}{}", f.rule);
        if seen_rules.insert(id) {
            rules.push(rule_descriptor(prefix, f));
        }
    }

    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "apo",
                    "version": pack.apo_version,
                    "informationUri": "https://github.com/baoulo/apo",
                    "rules": rules
                }
            },
            "results": results,
            "properties": {
                "evidence_schema": pack.evidence_schema,
                "repository": pack.repository
            }
        }]
    })
}

/// Write SARIF file.
pub fn write_sarif(pack: &EvidencePack, path: &Path) -> Result<()> {
    let doc = pack_to_sarif(pack);
    let text = serde_json::to_string_pretty(&doc)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}
