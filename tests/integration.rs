//! Integration tests for APO repository hygiene analysis.

use std::fs;
use std::path::Path;
use std::process::Command;

use apo::config::{Config, OutputFormat};
use apo::{analyze, analyze_and_write};
use tempfile::tempdir;

fn git(args: &[&str], cwd: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_AUTHOR_NAME", "APO Test")
        .env("GIT_AUTHOR_EMAIL", "apo@example.com")
        .env("GIT_COMMITTER_NAME", "APO Test")
        .env("GIT_COMMITTER_EMAIL", "apo@example.com")
        .status()
        .expect("git must be available");
    assert!(status.success(), "git {args:?} failed");
}

fn init_repo(root: &Path) {
    git(&["init"], root);
    // Avoid depending on global git identity in CI/dev machines.
    git(&["config", "user.email", "apo@example.com"], root);
    git(&["config", "user.name", "APO Test"], root);
}

fn write(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn commit_all(root: &Path, message: &str) {
    git(&["add", "-A"], root);
    git(&["commit", "-m", message], root);
}

#[test]
fn minimal_repo_reports_missing_controls() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    write(root, "src/main.rs", "fn main() {}\n");
    commit_all(root, "chore: initial commit");

    let report = analyze(&Config {
        target: root.display().to_string(),
        format: OutputFormat::Json,
        output: None,
        commit_sample_limit: 50,
        ..Config::default()
    })
    .unwrap();

    assert!(!report.findings.is_empty());
    let readme = report
        .findings
        .iter()
        .find(|f| f.rule == "documentation.readme")
        .expect("readme rule");
    assert_eq!(readme.status, apo::evidence::Status::Missing);

    let gitignore = report
        .findings
        .iter()
        .find(|f| f.rule == "local_development.gitignore")
        .expect("gitignore rule");
    assert_eq!(gitignore.status, apo::evidence::Status::Missing);

    assert!(!report.missing_controls.is_empty());
}

#[test]
fn well_hygiened_repo_detects_controls() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_repo(root);

    write(
        root,
        "README.md",
        "# Demo\n\n## Quickstart\n\n```bash\ncargo run\n```\n",
    );
    write(root, "CONTRIBUTING.md", "# Contributing\n");
    write(
        root,
        "SECURITY.md",
        "# Security\nReport issues privately.\n",
    );
    write(root, "LICENSE", "MIT\n");
    write(root, "ARCHITECTURE.md", "# Architecture\n");
    write(
        root,
        "docs/adr/0001-record-architecture-decisions.md",
        "# ADR 1\n",
    );
    write(root, "docs/runbooks/restart.md", "# Restart\n");
    write(root, ".github/CODEOWNERS", "* @owners\n");
    write(
        root,
        ".github/pull_request_template.md",
        "## Summary\nFixes #\n",
    );
    write(
        root,
        ".github/ISSUE_TEMPLATE/bug.md",
        "---\nname: Bug\n---\n",
    );
    write(root, ".gitignore", "/target\n");
    write(root, ".editorconfig", "root = true\n");
    write(root, "rustfmt.toml", "edition = \"2024\"\n");
    write(root, "clippy.toml", "avoid-breaking-exported-api = false\n");
    write(root, ".pre-commit-config.yaml", "repos: []\n");
    write(root, "Makefile", "setup:\n\t@echo setup\n");
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    write(
        root,
        "tests/smoke.rs",
        "#[test]\nfn works() { assert_eq!(demo::add(1, 2), 3); }\n",
    );
    write(
        root,
        ".github/workflows/ci.yml",
        r#"name: ci
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo llvm-cov
"#,
    );
    write(
        root,
        ".github/workflows/release.yml",
        r#"name: release
on:
  push:
    tags: ["v*"]
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo publish
"#,
    );
    write(
        root,
        ".github/dependabot.yml",
        "version: 2\nupdates:\n  - package-ecosystem: cargo\n    directory: /\n    schedule:\n      interval: weekly\n",
    );
    write(root, "deny.toml", "[advisories]\n");
    write(root, ".gitleaks.toml", "title = \"gitleaks\"\n");
    write(
        root,
        "codecov.yml",
        "coverage:\n  status:\n    project:\n      default:\n        target: 80%\n",
    );
    write(
        root,
        ".commitlintrc.json",
        "{ \"extends\": [\"@commitlint/config-conventional\"] }\n",
    );

    commit_all(root, "feat: initial well-hygiened repository");

    let out = root.join("out");
    fs::create_dir_all(&out).unwrap();

    let (report, written) = analyze_and_write(&Config {
        target: root.display().to_string(),
        format: OutputFormat::Both,
        output: Some(out.clone()),
        commit_sample_limit: 50,
        ..Config::default()
    })
    .unwrap();

    assert_eq!(written.len(), 3);
    let prefix = report.artifact_prefix();
    assert!(
        out.join(format!("{prefix}-repository-hygiene.md"))
            .is_file()
    );
    assert!(
        out.join(format!("{prefix}-repository-hygiene.json"))
            .is_file()
    );
    assert!(
        out.join(format!("{prefix}-repository-hygiene-badge.svg"))
            .is_file()
    );

    let readme = report
        .findings
        .iter()
        .find(|f| f.rule == "documentation.readme")
        .unwrap();
    assert_eq!(readme.status, apo::evidence::Status::Enforced);

    let ci = report
        .findings
        .iter()
        .find(|f| f.rule == "delivery.ci_workflows")
        .unwrap();
    assert_eq!(ci.status, apo::evidence::Status::Present);

    let overall = report.policy.overall_score.expect("overall score");
    assert!(
        overall > 60.0,
        "expected well-hygiened score > 60, got {overall}"
    );
}

#[test]
fn rejects_non_git_path() {
    let dir = tempdir().unwrap();
    let err = analyze(&Config {
        target: dir.path().display().to_string(),
        ..Config::default()
    })
    .unwrap_err();
    assert!(matches!(err, apo::Error::NotAGitRepository(_)));
}

#[test]
fn analyzes_remote_file_uri() {
    use std::process::Command;

    let bare_parent = tempdir().unwrap();
    let seed = tempdir().unwrap();
    let bare = bare_parent.path().join("demo.git");

    // Seed a commit, then create a bare remote we can clone via file://
    init_repo(seed.path());
    write(seed.path(), "README.md", "# remote demo\n\n## Quickstart\n");
    write(seed.path(), "LICENSE", "MIT\n");
    write(seed.path(), ".gitignore", "/target\n");
    commit_all(seed.path(), "feat: seed remote");

    let status = Command::new("git")
        .args([
            "clone",
            "--bare",
            seed.path().to_str().unwrap(),
            bare.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let uri = format!("file://{}", bare.display());
    let out = tempdir().unwrap();
    let (report, written) = analyze_and_write(&Config {
        target: uri.clone(),
        format: OutputFormat::Json,
        output: Some(out.path().join("remote.json")),
        commit_sample_limit: 20,
        ..Config::default()
    })
    .unwrap();

    assert_eq!(written.len(), 2);
    assert_eq!(report.repository, uri);
    assert_eq!(report.source_uri.as_deref(), Some(uri.as_str()));
    assert!(report.checkout_path.is_some());
    assert!(written.iter().any(|p| {
        p.extension().and_then(|e| e.to_str()) == Some("svg")
            || p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("-badge.svg"))
    }));

    let readme = report
        .findings
        .iter()
        .find(|f| f.rule == "documentation.readme")
        .unwrap();
    assert_eq!(readme.status, apo::evidence::Status::Enforced);
}

#[test]
fn elixir_pack_detects_mix_tooling() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_repo(root);

    write(
        root,
        "mix.exs",
        r#"
defmodule Demo.MixProject do
  use Mix.Project
  def project, do: [app: :demo, version: "0.1.0", aliases: aliases()]
  defp aliases do
    [
      check: [
        "format --check-formatted",
        "compile --warnings-as-errors",
        "credo --strict",
        "dialyzer",
        "hex.audit",
        "sobelow --config",
        "doctor --full",
        "docs --warnings-as-errors"
      ]
    ]
  end
end
"#,
    );
    write(
        root,
        ".formatter.exs",
        "[\n  inputs: [\"{mix,.formatter}.exs\", \"{config,lib,test}/**/*.{ex,exs}\"]\n]\n",
    );
    write(root, ".credo.exs", "%{configs: [%{name: \"default\"}]}\n");
    write(
        root,
        ".github/workflows/ci.yml",
        "name: ci\non: push\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: mix format --check-formatted\n      - run: mix credo --strict\n      - run: mix hex.audit\n      - run: mix dialyzer\n      - run: mix sobelow --config\n      - run: mix doctor --full\n      - run: mix docs --warnings-as-errors\n      - run: mix compile --warnings-as-errors\n",
    );
    write(root, "README.md", "# Demo\n");
    commit_all(root, "feat: elixir hygiene fixtures");

    let report = analyze(&Config {
        target: root.display().to_string(),
        format: OutputFormat::Json,
        commit_sample_limit: 20,
        ..Config::default()
    })
    .unwrap();

    let packs = apo::packs::for_repo(&apo::discovery::discover(root, 20).unwrap());
    assert!(
        packs.active_packs.iter().any(|p| p == "elixir"),
        "expected elixir pack, got {:?}",
        packs.active_packs
    );

    let fmt = report
        .findings
        .iter()
        .find(|f| f.rule == "local_development.formatter")
        .unwrap();
    assert!(
        matches!(
            fmt.status,
            apo::evidence::Status::Present | apo::evidence::Status::Enforced
        ),
        "formatter {:?}",
        fmt.status
    );

    let lint = report
        .findings
        .iter()
        .find(|f| f.rule == "local_development.linter")
        .unwrap();
    assert!(
        matches!(
            lint.status,
            apo::evidence::Status::Present | apo::evidence::Status::Enforced
        ),
        "linter {:?}",
        lint.status
    );

    let deps = report
        .findings
        .iter()
        .find(|f| f.rule == "security.dependency_scanning")
        .unwrap();
    assert_eq!(deps.status, apo::evidence::Status::Enforced);

    let docs = report
        .findings
        .iter()
        .find(|f| f.rule == "documentation.doc_tooling")
        .unwrap();
    assert_eq!(docs.status, apo::evidence::Status::Enforced);
}

#[test]
fn evidence_analyzer_detects_knowledge_and_ai() {
    use apo::config::AnalyzerMode;
    use apo::{analyze_evidence, evidence_and_write};

    let dir = tempdir().unwrap();
    let root = dir.path();
    init_repo(root);

    write(
        root,
        "README.md",
        "# Demo\n\nSee [Architecture](docs/ARCHITECTURE.md).\n",
    );
    write(
        root,
        "docs/ARCHITECTURE.md",
        "# Architecture\n\nCore modules.\n",
    );
    write(
        root,
        "docs/adr/0001-use-adrs.md",
        "# ADR 0001\n\n## Status\nAccepted\n",
    );
    write(root, "docs/runbooks/deploy.md", "# Deploy\n\nSteps.\n");
    write(root, "CODEOWNERS", "* @owners\n");
    write(root, "prompts/review.md", "# Review prompt\nv1\n");
    write(root, "AGENTS.md", "# Agents\nRoles for coding agents.\n");
    write(
        root,
        "docs/ai-governance.md",
        "# AI governance\n\nHuman review required for merges.\n",
    );
    write(
        root,
        ".github/workflows/ai-review.yml",
        "name: ai-review\non: pull_request\njobs:\n  review:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo review\n",
    );
    write(root, "src/lib.rs", "pub fn hello() {}\n");
    commit_all(root, "feat: knowledge and ai evidence fixtures");

    let report = analyze_evidence(&Config {
        target: root.display().to_string(),
        format: OutputFormat::Json,
        mode: AnalyzerMode::Evidence,
        ..Config::default()
    })
    .unwrap();

    assert_eq!(report.analyzer, "repository-evidence");
    assert!(!report.knowledge.artifacts.is_empty());
    assert!(report.knowledge.maturity_score > 0.0);
    assert!(report.ai.prompt_count >= 1);
    assert!(report.ai.governance_present);
    assert!(!report.executive_summary.is_empty());

    let out = tempdir().unwrap();
    let (written_report, paths) = evidence_and_write(&Config {
        target: root.display().to_string(),
        format: OutputFormat::Both,
        output: Some(out.path().to_path_buf()),
        mode: AnalyzerMode::Evidence,
        ..Config::default()
    })
    .unwrap();

    assert_eq!(paths.len(), 3);
    assert!(paths.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("-repository-evidence.md"))
    }));
    assert!(paths.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("-repository-evidence.json"))
    }));
    assert!(paths.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("-repository-evidence-badge.svg"))
    }));
    assert!(written_report.knowledge_maturity >= report.knowledge_maturity - 0.1);
}

#[test]
fn external_pack_from_apo_packs_dir_activates() {
    use apo::evidence::Status;
    use apo::packs::{load_packs_from_dir, merge_pack_catalog};

    let dir = tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    write(root, "README.md", "# crystal app\n");
    write(root, "shard.yml", "name: demo\n");
    write(root, "src/main.cr", "puts \"hi\"\n");
    write(root, ".ameba.yml", "Rules:\n");
    write(
        root,
        ".apo/packs/crystal.toml",
        r#"
id = "crystal"
kind = "language"
manifests = ["shard.yml"]
path_contains = [".cr"]

[[tooling]]
id = "lint"
maps_to = "local_development.linter"
configs = [".ameba.yml"]
ci_commands = ["ameba"]
"#,
    );
    commit_all(root, "feat: crystal with external pack");

    let report = analyze(&Config {
        target: root.display().to_string(),
        format: OutputFormat::Json,
        badge: false,
        ..Config::default()
    })
    .unwrap();

    let linter = report
        .findings
        .iter()
        .find(|f| f.rule == "local_development.linter")
        .expect("linter finding");
    assert!(
        matches!(linter.status, Status::Present | Status::Enforced),
        "expected Present/Enforced from external crystal pack, got {:?}",
        linter.status
    );

    // Conflict with builtin id must error
    let conflict_dir = tempdir().unwrap();
    fs::write(
        conflict_dir.path().join("rust.toml"),
        r#"
id = "rust"
kind = "language"
manifests = ["Cargo.toml"]
"#,
    )
    .unwrap();
    let external = load_packs_from_dir(conflict_dir.path()).unwrap();
    let err = merge_pack_catalog(external).unwrap_err();
    assert!(err.to_string().contains("conflicts"), "{err}");
}
