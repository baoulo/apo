//! Local development control rules.

use crate::discovery::RepoContext;
use crate::evidence::{Category, Confidence, EvidenceItem, Finding, Status};
use crate::rules::Rule;
use crate::rules::helpers;

pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(Gitignore),
        Box::new(Editorconfig),
        Box::new(Formatter),
        Box::new(Linter),
        Box::new(TypeChecker),
        Box::new(PreCommitHooks),
        Box::new(DevContainer),
    ]
}

struct Gitignore;
impl Rule for Gitignore {
    fn id(&self) -> &'static str {
        "local_development.gitignore"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        helpers::file_presence(
            ctx,
            self.id(),
            Category::LocalDevelopment,
            &[".gitignore"],
            ".gitignore detected.",
            "No .gitignore detected.",
            "Add a .gitignore appropriate for this project's artifacts.",
        )
    }
}

struct Editorconfig;
impl Rule for Editorconfig {
    fn id(&self) -> &'static str {
        "local_development.editorconfig"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        helpers::file_presence(
            ctx,
            self.id(),
            Category::LocalDevelopment,
            &[".editorconfig"],
            ".editorconfig detected.",
            "No .editorconfig detected.",
            "Add .editorconfig for consistent editor defaults.",
        )
    }
}

struct Formatter;
impl Rule for Formatter {
    fn id(&self) -> &'static str {
        "local_development.formatter"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut legacy = Vec::new();
        if let Some(pkg) = ctx.read_text("package.json") {
            let l = pkg.to_ascii_lowercase();
            if l.contains("\"prettier\"") || l.contains("\"format\"") {
                legacy.push(EvidenceItem::path("package.json"));
            }
        }
        if let Some(py) = ctx.read_text("pyproject.toml") {
            let l = py.to_ascii_lowercase();
            if l.contains("[tool.black]") || l.contains("[tool.ruff") || l.contains("[tool.isort]")
            {
                legacy.push(EvidenceItem::path_detail(
                    "pyproject.toml",
                    "formatter tool table",
                ));
            }
        }
        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::LocalDevelopment,
            crate::packs::MapsTo::Formatter,
            legacy,
            "No formatter configuration detected.",
            "Formatter configuration detected.",
            "Formatter configuration and CI/script enforcement detected.",
            "Add formatter config and gate it in CI (e.g. mix format --check-formatted, cargo fmt, prettier).",
        )
    }
}

struct Linter;
impl Rule for Linter {
    fn id(&self) -> &'static str {
        "local_development.linter"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut legacy = Vec::new();
        if let Some(pkg) = ctx.read_text("package.json") {
            let l = pkg.to_ascii_lowercase();
            if l.contains("eslint") || l.contains("\"lint\"") {
                legacy.push(EvidenceItem::path("package.json"));
            }
        }
        if let Some(py) = ctx.read_text("pyproject.toml") {
            let l = py.to_ascii_lowercase();
            if l.contains("[tool.ruff") || l.contains("[tool.pylint") || l.contains("flake8") {
                legacy.push(EvidenceItem::path("pyproject.toml"));
            }
        }
        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::LocalDevelopment,
            crate::packs::MapsTo::Linter,
            legacy,
            "No linter configuration detected.",
            "Linter configuration detected.",
            "Linter configuration and CI/script enforcement detected.",
            "Add linter config and gate it in CI (e.g. mix credo --strict, eslint, clippy).",
        )
    }
}

struct TypeChecker;
impl Rule for TypeChecker {
    fn id(&self) -> &'static str {
        "local_development.type_checker"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let signals = ctx.detect_signals();
        let mut legacy = Vec::new();
        if signals.has_cargo {
            legacy.push(EvidenceItem::path("Cargo.toml"));
        }
        if signals.has_go_mod {
            legacy.push(EvidenceItem::path("go.mod"));
        }
        if signals.has_mix {
            // Dialyzer / gradual typing — Presence of mix alone is not enough;
            // packs supply dialyzer configs/CI. Keep mix.exs only as weak context via packs.
        }
        if let Some(py) = ctx.read_text("pyproject.toml") {
            let l = py.to_ascii_lowercase();
            if l.contains("[tool.mypy]") || l.contains("[tool.pyright]") || l.contains("mypy") {
                legacy.push(EvidenceItem::path("pyproject.toml"));
            }
        }
        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::LocalDevelopment,
            crate::packs::MapsTo::TypeChecker,
            legacy,
            "No type checker configuration detected.",
            "Type checker / typed toolchain configuration detected.",
            "Type checking gated in CI/scripts.",
            "Add type checking (dialyzer, tsc, mypy, or compile --warnings-as-errors) and gate it in CI.",
        )
    }
}

struct PreCommitHooks;
impl Rule for PreCommitHooks {
    fn id(&self) -> &'static str {
        "local_development.pre_commit_hooks"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut hits = Vec::new();
        for name in [
            ".pre-commit-config.yaml",
            ".pre-commit-config.yml",
            ".husky/pre-commit",
            "lefthook.yml",
            "lefthook.yaml",
            ".lefthook.yml",
        ] {
            if ctx.has_file(name) {
                hits.push(name.to_string());
            }
        }
        if ctx.has_dir(".husky") {
            hits.push(".husky".into());
        }
        if let Some(pkg) = ctx.read_text("package.json") {
            let l = pkg.to_ascii_lowercase();
            if l.contains("husky") || l.contains("lint-staged") || l.contains("simple-git-hooks") {
                hits.push("package.json".into());
            }
        }

        hits.sort();
        hits.dedup();

        if hits.is_empty() {
            Finding::builder(self.id(), Category::LocalDevelopment)
                .status(Status::Missing)
                .confidence(Confidence::High)
                .summary("No pre-commit hook configuration detected.")
                .remediation("Add pre-commit, husky, or lefthook hook configuration.")
                .build()
        } else {
            let mut b = Finding::builder(self.id(), Category::LocalDevelopment)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Pre-commit hook configuration detected.");
            for h in hits {
                b = b.push_evidence(EvidenceItem::path(h));
            }
            b.build()
        }
    }
}

struct DevContainer;
impl Rule for DevContainer {
    fn id(&self) -> &'static str {
        "local_development.dev_environment"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut hits = Vec::new();
        for name in [
            ".devcontainer/devcontainer.json",
            "devcontainer.json",
            "Dockerfile",
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
            "Makefile",
            "justfile",
            "Justfile",
            "scripts/setup.sh",
            "script/setup",
            "bin/setup",
            "setup.sh",
            "CONTRIBUTING.md",
        ] {
            if ctx.has_file(name) {
                hits.push(name.to_string());
            }
        }
        if ctx.has_dir(".devcontainer") {
            hits.push(".devcontainer".into());
        }

        // Prefer stronger signals over CONTRIBUTING alone
        let strong: Vec<_> = hits
            .iter()
            .filter(|h| !h.eq_ignore_ascii_case("CONTRIBUTING.md"))
            .cloned()
            .collect();

        if strong.is_empty() && hits.is_empty() {
            Finding::builder(self.id(), Category::LocalDevelopment)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No dev container or setup scripts detected.")
                .remediation(
                    "Add .devcontainer, Dockerfile, or setup scripts for local onboarding.",
                )
                .build()
        } else if strong.is_empty() {
            Finding::builder(self.id(), Category::LocalDevelopment)
                .status(Status::Partial)
                .confidence(Confidence::Low)
                .summary("Only contribution docs found; no explicit setup automation detected.")
                .push_evidence(EvidenceItem::path("CONTRIBUTING.md"))
                .remediation("Add .devcontainer or setup scripts for reproducible local setup.")
                .build()
        } else {
            let mut b = Finding::builder(self.id(), Category::LocalDevelopment)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Dev environment / setup automation detected.");
            for h in strong {
                b = b.push_evidence(EvidenceItem::path(h));
            }
            b.build()
        }
    }
}
