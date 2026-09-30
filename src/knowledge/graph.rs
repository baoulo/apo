//! Knowledge graph linking docs, code, tests, ADRs, and runbooks.

use serde::{Deserialize, Serialize};

use crate::discovery::RepoContext;
use crate::knowledge::{KnowledgeArtifact, KnowledgeKind};

/// Kind of node in the knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Document,
    Source,
    Test,
    Adr,
    Runbook,
    Prompt,
    ApiDoc,
    Diagram,
}

/// Kind of edge between knowledge nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Markdown / path reference.
    References,
    /// Doc appears to document a code area (heuristic path affinity).
    Documents,
    /// Test path related to source.
    Tests,
}

/// Graph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub path: String,
    pub kind: NodeKind,
}

/// Graph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Deterministic knowledge graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

/// Build a knowledge graph from inventory + discovered artifacts.
pub fn build_graph(ctx: &RepoContext, artifacts: &[KnowledgeArtifact]) -> KnowledgeGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    let push_node = |nodes: &mut Vec<KnowledgeNode>,
                     seen: &mut std::collections::BTreeSet<String>,
                     path: &str,
                     kind: NodeKind| {
        if seen.insert(path.to_string()) {
            nodes.push(KnowledgeNode {
                id: path.to_string(),
                path: path.to_string(),
                kind,
            });
        }
    };

    for a in artifacts {
        let kind = match a.kind {
            KnowledgeKind::Adr => NodeKind::Adr,
            KnowledgeKind::Runbook => NodeKind::Runbook,
            KnowledgeKind::PromptLibrary => NodeKind::Prompt,
            KnowledgeKind::Api => NodeKind::ApiDoc,
            KnowledgeKind::Diagram => NodeKind::Diagram,
            _ => NodeKind::Document,
        };
        push_node(&mut nodes, &mut seen, &a.path, kind);
    }

    // Sample source and test files for linking
    let sources: Vec<_> = ctx
        .inventory
        .iter()
        .filter(|e| is_source(&e.relative))
        .take(80)
        .map(|e| e.relative.clone())
        .collect();
    let tests: Vec<_> = ctx
        .inventory
        .iter()
        .filter(|e| is_test(&e.relative))
        .take(40)
        .map(|e| e.relative.clone())
        .collect();

    for s in &sources {
        push_node(&mut nodes, &mut seen, s, NodeKind::Source);
    }
    for t in &tests {
        push_node(&mut nodes, &mut seen, t, NodeKind::Test);
    }

    // Reference edges from markdown link scan (lightweight: path substrings in doc content)
    for a in artifacts {
        let Some(content) = ctx.read_text(&a.path) else {
            continue;
        };
        for s in &sources {
            let base = std::path::Path::new(s)
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if base.len() >= 4 && content.contains(base) {
                edges.push(KnowledgeEdge {
                    from: a.path.clone(),
                    to: s.clone(),
                    kind: EdgeKind::Documents,
                    detail: Some(format!("doc mentions `{base}`")),
                });
            }
            if content.contains(s) {
                edges.push(KnowledgeEdge {
                    from: a.path.clone(),
                    to: s.clone(),
                    kind: EdgeKind::References,
                    detail: None,
                });
            }
        }
    }

    // Test → source affinity by stem
    for t in &tests {
        let stem = std::path::Path::new(t)
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .trim_end_matches("_test")
            .trim_end_matches("_spec")
            .trim_end_matches(".test")
            .trim_end_matches(".spec");
        for s in &sources {
            let s_stem = std::path::Path::new(s)
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if !stem.is_empty() && (s_stem == stem || s.contains(stem)) {
                edges.push(KnowledgeEdge {
                    from: t.clone(),
                    to: s.clone(),
                    kind: EdgeKind::Tests,
                    detail: None,
                });
            }
        }
    }

    // Cap edges for report size
    edges.sort_by(|a, b| (&a.from, &a.to).cmp(&(&b.from, &b.to)));
    edges.dedup_by(|a, b| a.from == b.from && a.to == b.to && a.kind == b.kind);
    if edges.len() > 200 {
        edges.truncate(200);
    }

    KnowledgeGraph { nodes, edges }
}

fn is_source(path: &str) -> bool {
    let l = path.to_ascii_lowercase();
    if is_test(&l) {
        return false;
    }
    l.ends_with(".rs")
        || l.ends_with(".go")
        || l.ends_with(".py")
        || l.ends_with(".ts")
        || l.ends_with(".tsx")
        || l.ends_with(".js")
        || l.ends_with(".jsx")
        || l.ends_with(".java")
        || l.ends_with(".kt")
        || l.ends_with(".ex")
        || l.ends_with(".exs")
        || l.ends_with(".rb")
}

fn is_test(path: &str) -> bool {
    let l = path.to_ascii_lowercase();
    l.contains("/test/")
        || l.contains("/tests/")
        || l.contains("/__tests__/")
        || l.contains("_test.")
        || l.contains(".test.")
        || l.contains(".spec.")
        || l.contains("_spec.")
}
