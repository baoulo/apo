//! Baseline diffs for unified evidence packs.

use crate::error::{Error, Result};
use crate::evidence::Status;
use crate::report::pack::{CategoryDelta, EVIDENCE_SCHEMA, EvidencePack, PackDiff};
use std::collections::BTreeSet;
use std::path::Path;

/// Load a prior pack JSON and compute diffs against `current`.
pub fn apply_baseline(current: &mut EvidencePack, baseline_path: &Path) -> Result<()> {
    let text = std::fs::read_to_string(baseline_path).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("read baseline {}: {e}", baseline_path.display()),
        ))
    })?;
    let prior: EvidencePack = serde_json::from_str(&text).map_err(|e| {
        Error::Config(format!(
            "invalid baseline JSON {}: {e}",
            baseline_path.display()
        ))
    })?;

    if prior.evidence_schema != EVIDENCE_SCHEMA && !prior.evidence_schema.starts_with("apo-v0.") {
        return Err(Error::Config(format!(
            "incompatible baseline schema '{}' (expected {EVIDENCE_SCHEMA})",
            prior.evidence_schema
        )));
    }

    current.diff = Some(compute_diff(&prior, current));
    Ok(())
}

fn compute_diff(prior: &EvidencePack, current: &EvidencePack) -> PackDiff {
    let before_h = prior.hygiene.policy.overall_score;
    let after_h = current.hygiene.policy.overall_score;
    let hygiene_score_delta = match (before_h, after_h) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };

    let mut category_deltas = Vec::new();
    for cat in crate::evidence::Category::all() {
        let before = prior
            .hygiene
            .policy
            .categories
            .iter()
            .find(|c| c.category == *cat)
            .and_then(|c| c.score);
        let after = current
            .hygiene
            .policy
            .categories
            .iter()
            .find(|c| c.category == *cat)
            .and_then(|c| c.score);
        let delta = match (before, after) {
            (Some(a), Some(b)) => Some(b - a),
            _ => None,
        };
        category_deltas.push(CategoryDelta {
            category: cat.id().to_string(),
            before,
            after,
            delta,
        });
    }

    let prior_gaps = gap_ids(prior);
    let current_gaps = gap_ids(current);
    let newly_gapped: Vec<_> = current_gaps.difference(&prior_gaps).cloned().collect();
    let newly_satisfied: Vec<_> = prior_gaps.difference(&current_gaps).cloned().collect();
    let unchanged_gaps: Vec<_> = prior_gaps.intersection(&current_gaps).cloned().collect();

    let km = current.evidence.knowledge_maturity - prior.evidence.knowledge_maturity;
    let am = current.evidence.ai_maturity - prior.evidence.ai_maturity;

    let summary = format!(
        "Hygiene score delta {}; knowledge {km:+.1}; AI {am:+.1}. \
         Newly gapped: {}; newly satisfied: {}; unchanged gaps: {}.",
        hygiene_score_delta
            .map(|d| format!("{d:+.1}"))
            .unwrap_or_else(|| "n/a".into()),
        newly_gapped.len(),
        newly_satisfied.len(),
        unchanged_gaps.len(),
    );

    PackDiff {
        baseline_schema: prior.evidence_schema.clone(),
        baseline_generated_at: Some(prior.generated_at.clone()),
        hygiene_score_delta,
        category_deltas,
        knowledge_maturity_delta: km,
        ai_maturity_delta: am,
        newly_gapped,
        newly_satisfied,
        unchanged_gaps,
        summary,
    }
}

fn gap_ids(pack: &EvidencePack) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for f in &pack.hygiene.findings {
        if matches!(
            f.status,
            Status::Missing | Status::Partial | Status::Unknown
        ) {
            ids.insert(format!("hygiene:{}", f.rule));
        }
    }
    for f in &pack.evidence.knowledge.findings {
        if matches!(
            f.status,
            Status::Missing | Status::Partial | Status::Unknown
        ) {
            ids.insert(format!("knowledge:{}", f.rule));
        }
    }
    for f in &pack.evidence.ai.findings {
        if matches!(
            f.status,
            Status::Missing | Status::Partial | Status::Unknown
        ) {
            ids.insert(format!("ai:{}", f.rule));
        }
    }
    for k in &pack.evidence.knowledge.kinds_missing {
        ids.insert(format!("kind:{}", k.as_str()));
    }
    ids
}

/// Render a short markdown summary of a pack diff.
pub fn render_diff_markdown(diff: &PackDiff) -> String {
    let mut out = String::from("# APO evidence pack baseline diff\n\n");
    out.push_str(&format!("{}\n\n", diff.summary));
    if let Some(d) = diff.hygiene_score_delta {
        out.push_str(&format!("- Hygiene overall delta: {d:+.1}\n"));
    }
    out.push_str(&format!(
        "- Knowledge maturity delta: {:+.1}\n- AI maturity delta: {:+.1}\n\n",
        diff.knowledge_maturity_delta, diff.ai_maturity_delta
    ));
    if !diff.newly_satisfied.is_empty() {
        out.push_str("## Newly satisfied\n\n");
        for id in &diff.newly_satisfied {
            out.push_str(&format!("- `{id}`\n"));
        }
        out.push('\n');
    }
    if !diff.newly_gapped.is_empty() {
        out.push_str("## Newly gapped\n\n");
        for id in &diff.newly_gapped {
            out.push_str(&format!("- `{id}`\n"));
        }
        out.push('\n');
    }
    if !diff.unchanged_gaps.is_empty() {
        out.push_str("## Unchanged gaps\n\n");
        for id in diff.unchanged_gaps.iter().take(40) {
            out.push_str(&format!("- `{id}`\n"));
        }
        if diff.unchanged_gaps.len() > 40 {
            out.push_str(&format!(
                "- … and {} more\n",
                diff.unchanged_gaps.len() - 40
            ));
        }
    }
    out
}
