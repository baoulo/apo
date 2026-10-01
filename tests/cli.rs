//! CLI smoke tests.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::process::Command;

fn init_tiny_repo(root: &std::path::Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "apo@example.com"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "APO Test"])
        .current_dir(root)
        .status()
        .unwrap();
    fs::write(root.join("README.md"), "# tiny\n").unwrap();
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: init"])
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "APO Test")
        .env("GIT_AUTHOR_EMAIL", "apo@example.com")
        .env("GIT_COMMITTER_NAME", "APO Test")
        .env("GIT_COMMITTER_EMAIL", "apo@example.com")
        .status()
        .unwrap();
}

#[test]
fn analyze_json_writes_and_prints() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());

    cargo_bin_cmd!("apo")
        .args([
            "analyze",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            dir.path().join("report.json").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("repository-hygiene"))
        .stdout(predicate::str::contains("documentation.readme"));

    let written = fs::read_to_string(dir.path().join("report.json")).unwrap();
    assert!(written.contains("\"analyzer\": \"repository-hygiene\""));
}

#[test]
fn prompt_command_writes_llm_remediation_file() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out = dir.path().join("prompt-out");
    fs::create_dir_all(&out).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "prompt",
            dir.path().to_str().unwrap(),
            "--output",
            out.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let entries: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let prompt_name = entries
        .iter()
        .find(|n| n.ends_with("-repository-hygiene-prompt.md"))
        .expect("expected prefixed prompt file");
    let prompt = fs::read_to_string(out.join(prompt_name)).unwrap();
    assert!(prompt.contains("Repository hygiene remediation task"));
    assert!(prompt.contains("Gaps to remediate"));
    assert!(prompt.contains("documentation.license") || prompt.contains("local_development"));
}

#[test]
fn evidence_json_writes_report() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out = dir.path().join("evidence-out");
    fs::create_dir_all(&out).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "evidence",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("repository-evidence"))
        .stdout(predicate::str::contains("knowledge"));

    let entries: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let json_name = entries
        .iter()
        .find(|n| n.ends_with("-repository-evidence.json"))
        .expect("expected prefixed evidence json");
    let written = fs::read_to_string(out.join(json_name)).unwrap();
    assert!(written.contains("\"analyzer\": \"repository-evidence\""));
    assert!(written.contains("knowledge_maturity"));
    assert!(written.contains("ai_maturity"));
}

#[test]
fn evidence_llm_prompt_writes_file() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out = dir.path().join("evidence-prompt-out");
    fs::create_dir_all(&out).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "evidence",
            dir.path().to_str().unwrap(),
            "--llm-prompt",
            "--output",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let entries: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let prompt_name = entries
        .iter()
        .find(|n| n.ends_with("-repository-evidence-prompt.md"))
        .expect("expected prefixed evidence prompt file");
    let prompt = fs::read_to_string(out.join(prompt_name)).unwrap();
    assert!(prompt.contains("knowledge & AI evidence remediation"));
    assert!(prompt.contains("Gaps to remediate") || prompt.contains("gaps to remediate"));
    assert!(prompt.contains("knowledge.") || prompt.contains("ai."));
}

#[test]
fn analyze_writes_hygiene_badge_svg() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out = dir.path().join("badge-out");
    fs::create_dir_all(&out).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "analyze",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let entries: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let badge_name = entries
        .iter()
        .find(|n| n.ends_with("-repository-hygiene-badge.svg"))
        .expect("expected hygiene badge svg");
    let svg = fs::read_to_string(out.join(badge_name)).unwrap();
    assert!(svg.contains("apo hygiene"));
    assert!(svg.contains("<svg"));
}

#[test]
fn evidence_writes_badge_and_no_badge_skips() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out = dir.path().join("ev-badge-out");
    fs::create_dir_all(&out).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "evidence",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let entries: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let badge_name = entries
        .iter()
        .find(|n| n.ends_with("-repository-evidence-badge.svg"))
        .expect("expected evidence badge svg");
    let svg = fs::read_to_string(out.join(badge_name)).unwrap();
    assert!(svg.contains("knowledge"));
    assert!(svg.contains("ai"));

    let out2 = dir.path().join("ev-nobadge");
    fs::create_dir_all(&out2).unwrap();
    cargo_bin_cmd!("apo")
        .args([
            "evidence",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out2.to_str().unwrap(),
            "--no-badge",
        ])
        .assert()
        .success();
    let entries2: Vec<_> = fs::read_dir(&out2)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        entries2
            .iter()
            .all(|n| !n.ends_with("-repository-evidence-badge.svg")),
        "expected no badge when --no-badge: {entries2:?}"
    );
}

#[test]
fn report_writes_unified_pack_json_and_sarif() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out = dir.path().join("pack-out");
    fs::create_dir_all(&out).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "report",
            dir.path().to_str().unwrap(),
            "--format",
            "both",
            "--output",
            out.to_str().unwrap(),
            "--sarif",
            "--no-badge",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("apo-v0.3"));

    let entries: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        entries
            .iter()
            .any(|n| n.ends_with("-repository-evidence-pack.json")),
        "missing pack json: {entries:?}"
    );
    assert!(
        entries
            .iter()
            .any(|n| n.ends_with("-repository-evidence-pack.md")),
        "missing pack md: {entries:?}"
    );
    assert!(
        entries
            .iter()
            .any(|n| n.ends_with("-repository-evidence-pack.sarif")),
        "missing sarif: {entries:?}"
    );
    let json_path = entries
        .iter()
        .find(|n| n.ends_with("-repository-evidence-pack.json"))
        .unwrap();
    let body = fs::read_to_string(out.join(json_path)).unwrap();
    assert!(body.contains("\"evidence_schema\": \"apo-v0.3\""));
    assert!(body.contains("\"analyzer\": \"repository-evidence-pack\""));
}

#[test]
fn report_baseline_emits_diff() {
    let dir = tempdir().unwrap();
    init_tiny_repo(dir.path());
    let out1 = dir.path().join("b1");
    let out2 = dir.path().join("b2");
    fs::create_dir_all(&out1).unwrap();
    fs::create_dir_all(&out2).unwrap();

    cargo_bin_cmd!("apo")
        .args([
            "report",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out1.to_str().unwrap(),
            "--no-badge",
        ])
        .assert()
        .success();

    let baseline = fs::read_dir(&out1)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("-repository-evidence-pack.json"))
        })
        .expect("baseline json");

    cargo_bin_cmd!("apo")
        .args([
            "report",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out2.to_str().unwrap(),
            "--baseline",
            baseline.to_str().unwrap(),
            "--no-badge",
        ])
        .assert()
        .success();

    let pack_path = fs::read_dir(&out2)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("-repository-evidence-pack.json"))
        })
        .unwrap();
    let body = fs::read_to_string(pack_path).unwrap();
    assert!(body.contains("\"diff\""));
    assert!(body.contains("hygiene_score_delta"));
    assert!(
        fs::read_dir(&out2)
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e
                .file_name()
                .to_string_lossy()
                .ends_with("-repository-evidence-pack-diff.md")),
        "expected diff markdown"
    );
}
