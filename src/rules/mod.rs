//! Hygiene rule engine.

mod collaboration;
mod delivery;
mod documentation;
mod local_dev;
mod security;
mod testing;

use crate::discovery::RepoContext;
use crate::evidence::Finding;

/// A hygiene rule that produces observational evidence.
pub trait Rule: Send + Sync {
    /// Stable rule id, e.g. `documentation.readme`.
    fn id(&self) -> &'static str;

    /// Evaluate the rule against the repository context.
    fn evaluate(&self, ctx: &RepoContext) -> Finding;
}

/// Run all built-in rules (in parallel) and return findings.
pub fn evaluate_all(ctx: &RepoContext) -> Vec<Finding> {
    use rayon::prelude::*;

    let rules = all_rules();
    let mut findings: Vec<Finding> = rules.par_iter().map(|rule| rule.evaluate(ctx)).collect();

    findings.sort_by(|a, b| a.rule.cmp(&b.rule));
    findings
}

/// Construct the v0.1 built-in rule set.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut rules: Vec<Box<dyn Rule>> = Vec::new();
    rules.extend(documentation::rules());
    rules.extend(local_dev::rules());
    rules.extend(testing::rules());
    rules.extend(security::rules());
    rules.extend(delivery::rules());
    rules.extend(collaboration::rules());
    rules
}

/// Shared helpers for path-based rules.
pub(crate) mod helpers {
    use crate::discovery::RepoContext;
    use crate::evidence::{Category, Confidence, EvidenceItem, Finding, Status};

    /// Evaluate presence of any of the candidate files.
    pub fn file_presence(
        ctx: &RepoContext,
        rule: &str,
        category: Category,
        candidates: &[&str],
        present_summary: &str,
        missing_summary: &str,
        remediation: &str,
    ) -> Finding {
        if let Some(path) = ctx.first_existing(candidates) {
            Finding::builder(rule, category)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary(present_summary)
                .push_evidence(EvidenceItem::path(path))
                .build()
        } else {
            Finding::builder(rule, category)
                .status(Status::Missing)
                .confidence(Confidence::High)
                .summary(missing_summary)
                .remediation(remediation)
                .build()
        }
    }

    /// Evaluate presence with optional content keyword enrichment.
    #[allow(clippy::too_many_arguments)]
    pub fn file_presence_with_keywords(
        ctx: &RepoContext,
        rule: &str,
        category: Category,
        candidates: &[&str],
        keywords: &[&str],
        enforced_summary: &str,
        present_summary: &str,
        missing_summary: &str,
        remediation: &str,
    ) -> Finding {
        let Some(path) = ctx.first_existing(candidates) else {
            return Finding::builder(rule, category)
                .status(Status::Missing)
                .confidence(Confidence::High)
                .summary(missing_summary)
                .remediation(remediation)
                .build();
        };

        let content = ctx.read_text(path).unwrap_or_default();
        let lower = content.to_ascii_lowercase();
        let hits: Vec<&str> = keywords
            .iter()
            .copied()
            .filter(|k| lower.contains(&k.to_ascii_lowercase()))
            .collect();

        if !hits.is_empty() {
            Finding::builder(rule, category)
                .status(Status::Enforced)
                .confidence(Confidence::High)
                .summary(enforced_summary)
                .push_evidence(EvidenceItem::path_detail(
                    path,
                    format!("matched keywords: {}", hits.join(", ")),
                ))
                .build()
        } else {
            Finding::builder(rule, category)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary(present_summary)
                .push_evidence(EvidenceItem::path(path))
                .build()
        }
    }

    /// Search CI workflow bodies for any of the needles.
    pub fn ci_mentions(ctx: &RepoContext, needles: &[&str]) -> Vec<EvidenceItem> {
        script_mentions(ctx, needles)
    }

    /// Search CI workflows and common script/config files for command needles.
    pub fn script_mentions(ctx: &RepoContext, needles: &[&str]) -> Vec<EvidenceItem> {
        if needles.is_empty() {
            return Vec::new();
        }
        let signals = ctx.detect_signals();
        let mut paths = signals.ci_workflow_paths.clone();
        for extra in [
            "Makefile",
            "makefile",
            "Justfile",
            "justfile",
            "mix.exs",
            "package.json",
            ".gitlab-ci.yml",
            "azure-pipelines.yml",
            "Jenkinsfile",
            "bitbucket-pipelines.yml",
            "Taskfile.yml",
            "Taskfile.yaml",
        ] {
            if ctx.has_file(extra) {
                paths.push(extra.to_string());
            }
        }
        paths.sort();
        paths.dedup();

        let mut items = Vec::new();
        for path in &paths {
            let Some(content) = ctx.read_text(path) else {
                continue;
            };
            let lower = content.to_ascii_lowercase();
            let matched: Vec<&str> = needles
                .iter()
                .copied()
                .filter(|n| lower.contains(&n.to_ascii_lowercase()))
                .collect();
            if !matched.is_empty() {
                items.push(EvidenceItem::path_detail(
                    path,
                    format!("mentions: {}", matched.join(", ")),
                ));
            }
        }
        items
    }

    /// Collect evidence from language/web packs for a hygiene rule mapping.
    pub fn pack_evidence(ctx: &RepoContext, maps_to: crate::packs::MapsTo) -> Vec<EvidenceItem> {
        let tooling = crate::packs::for_repo(ctx);
        let mut items = Vec::new();

        let config_names = tooling.config_names(maps_to);
        for path in find_configs(ctx, config_names.as_slice()) {
            items.push(EvidenceItem::path_detail(
                path,
                format!("pack config ({})", maps_to.rule_id()),
            ));
        }

        let cmds = tooling.ci_commands(maps_to);
        items.extend(script_mentions(ctx, cmds.as_slice()));

        // Content heuristics for pyproject / package.json already covered by packs' CI
        // and config names; also treat typed manifests as type-checker evidence.
        if maps_to == crate::packs::MapsTo::TypeChecker {
            for entry in tooling.entries_for(maps_to) {
                for cfg in &entry.configs {
                    if ctx.has_file(cfg)
                        && !items
                            .iter()
                            .any(|i| i.path.as_deref() == Some(cfg.as_str()))
                    {
                        items.push(EvidenceItem::path(cfg.clone()));
                    }
                }
            }
        }

        items.sort_by(|a, b| a.path.cmp(&b.path));
        items.dedup();
        items
    }

    /// Find config files by basename list.
    pub fn find_configs(ctx: &RepoContext, names: &[&str]) -> Vec<String> {
        if names.is_empty() {
            return Vec::new();
        }
        ctx.inventory
            .find_by_basenames(names)
            .into_iter()
            .map(|e| e.relative.clone())
            .collect()
    }

    /// Merge pack evidence with legacy hits; Enforced if any CI mention, else Present.
    #[allow(clippy::too_many_arguments)]
    pub fn finding_from_pack_or_legacy(
        ctx: &RepoContext,
        rule: &str,
        category: Category,
        maps_to: crate::packs::MapsTo,
        legacy_items: Vec<EvidenceItem>,
        missing_summary: &str,
        present_summary: &str,
        enforced_summary: &str,
        remediation: &str,
    ) -> Finding {
        let mut items = pack_evidence(ctx, maps_to);
        items.extend(legacy_items);
        items.sort_by(|a, b| a.path.cmp(&b.path));
        items.dedup();

        if items.is_empty() {
            return Finding::builder(rule, category)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary(missing_summary)
                .remediation(remediation)
                .build();
        }

        let enforced = items.iter().any(|i| {
            i.detail
                .as_deref()
                .is_some_and(|d| d.starts_with("mentions:"))
        });

        Finding::builder(rule, category)
            .status(if enforced {
                Status::Enforced
            } else {
                Status::Present
            })
            .confidence(Confidence::High)
            .summary(if enforced {
                enforced_summary
            } else {
                present_summary
            })
            .evidence(items)
            .build()
    }
}
