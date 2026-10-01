//! Testing & quality gate rules.

use crate::discovery::RepoContext;
use crate::evidence::{Category, Confidence, EvidenceItem, Finding, Status};
use crate::rules::Rule;
use crate::rules::helpers;

pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(TestFramework),
        Box::new(CoverageConfig),
        Box::new(CoverageEnforcement),
        Box::new(StaticAnalysisCi),
        Box::new(TypeCheckingCi),
        Box::new(PropertyTesting),
        Box::new(IntegrationTesting),
        Box::new(UiTesting),
    ]
}

struct TestFramework;
impl Rule for TestFramework {
    fn id(&self) -> &'static str {
        "testing.framework"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let signals = ctx.detect_signals();
        let mut hits = Vec::new();

        // Directory conventions
        for d in ["tests", "test", "spec", "__tests__", "src/test"] {
            if ctx.has_dir(d) {
                hits.push(format!("{d}/"));
            }
        }

        // Test file patterns
        let test_files = ctx.inventory.find_matching(|p| {
            let l = p.to_ascii_lowercase();
            l.contains("/test_")
                || l.contains("_test.")
                || l.contains(".test.")
                || l.contains(".spec.")
                || l.ends_with("_test.go")
                || l.ends_with("_test.rs")
                || l.ends_with("_spec.rb")
                || l.contains("/__tests__/")
        });
        for f in test_files.into_iter().take(15) {
            hits.push(f.relative.clone());
        }

        if let Some(pkg) = ctx.read_text("package.json") {
            let l = pkg.to_ascii_lowercase();
            if l.contains("jest")
                || l.contains("vitest")
                || l.contains("mocha")
                || l.contains("ava")
                || l.contains("\"test\"")
            {
                hits.push("package.json".into());
            }
        }
        if signals.has_cargo {
            hits.push("Cargo.toml".into());
        }
        if let Some(py) = ctx.read_text("pyproject.toml") {
            let l = py.to_ascii_lowercase();
            if l.contains("pytest") || l.contains("unittest") || l.contains("[tool.pytest") {
                hits.push("pyproject.toml".into());
            }
        }
        if ctx.has_file("pytest.ini") || ctx.has_file("tox.ini") {
            if ctx.has_file("pytest.ini") {
                hits.push("pytest.ini".into());
            }
            if ctx.has_file("tox.ini") {
                hits.push("tox.ini".into());
            }
        }

        hits.sort();
        hits.dedup();

        if hits.is_empty() {
            Finding::builder(self.id(), Category::Testing)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No test framework or test files detected.")
                .remediation("Add a test suite and framework configuration.")
                .build()
        } else {
            let mut b = Finding::builder(self.id(), Category::Testing)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Test framework or test files detected.");
            for h in hits.into_iter().take(12) {
                b = b.push_evidence(EvidenceItem::path(h));
            }
            b.build()
        }
    }
}

struct CoverageConfig;
impl Rule for CoverageConfig {
    fn id(&self) -> &'static str {
        "testing.coverage_config"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut hits = helpers::find_configs(
            ctx,
            &[
                "codecov.yml",
                "codecov.yaml",
                ".codecov.yml",
                "coverage.xml",
                ".coveragerc",
                "lcov.info",
            ],
        );
        if let Some(pkg) = ctx.read_text("package.json") {
            let l = pkg.to_ascii_lowercase();
            if l.contains("nyc")
                || l.contains("c8")
                || l.contains("istanbul")
                || l.contains("coverage")
            {
                hits.push("package.json".into());
            }
        }
        if let Some(py) = ctx.read_text("pyproject.toml") {
            let l = py.to_ascii_lowercase();
            if l.contains("coverage") || l.contains("pytest-cov") {
                hits.push("pyproject.toml".into());
            }
        }
        // CI mentions
        let ci = helpers::ci_mentions(
            ctx,
            &[
                "coverage",
                "codecov",
                "coveralls",
                "tarpaulin",
                "llvm-cov",
                "c8",
                "pytest-cov",
            ],
        );
        for item in &ci {
            if let Some(p) = &item.path {
                hits.push(p.clone());
            }
        }

        hits.sort();
        hits.dedup();

        if hits.is_empty() {
            Finding::builder(self.id(), Category::Testing)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No coverage configuration detected.")
                .remediation("Add coverage tooling config (codecov, tarpaulin, c8, pytest-cov).")
                .build()
        } else {
            let mut b = Finding::builder(self.id(), Category::Testing)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Coverage configuration detected.");
            for h in hits {
                b = b.push_evidence(EvidenceItem::path(h));
            }
            b.build()
        }
    }
}

struct CoverageEnforcement;
impl Rule for CoverageEnforcement {
    fn id(&self) -> &'static str {
        "testing.coverage_enforcement"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut items = helpers::ci_mentions(
            ctx,
            &[
                "coverage",
                "--fail-under",
                "codecov",
                "coveralls",
                "tarpaulin",
                "llvm-cov",
                "diff-cover",
            ],
        );

        // codecov / coveralls config often implies upload+gate intent
        for name in ["codecov.yml", "codecov.yaml", ".codecov.yml"] {
            if let Some(content) = ctx.read_text(name) {
                let l = content.to_ascii_lowercase();
                if l.contains("target") || l.contains("threshold") || l.contains("require") {
                    items.push(EvidenceItem::path_detail(
                        name,
                        "threshold/target configured",
                    ));
                } else {
                    items.push(EvidenceItem::path(name));
                }
            }
        }

        if items.is_empty() {
            Finding::builder(self.id(), Category::Testing)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No coverage enforcement signal detected in CI or config.")
                .remediation("Fail builds when coverage drops below a configured threshold.")
                .build()
        } else {
            let enforced = items.iter().any(|i| {
                i.detail.as_deref().is_some_and(|d| {
                    d.contains("threshold") || d.contains("fail-under") || d.contains("mentions")
                })
            });
            Finding::builder(self.id(), Category::Testing)
                .status(if enforced {
                    Status::Enforced
                } else {
                    Status::Partial
                })
                .confidence(Confidence::Medium)
                .summary(if enforced {
                    "Coverage enforcement signals detected."
                } else {
                    "Coverage tooling present; enforcement threshold unclear."
                })
                .evidence(items)
                .build()
        }
    }
}

struct StaticAnalysisCi;
impl Rule for StaticAnalysisCi {
    fn id(&self) -> &'static str {
        "testing.static_analysis_ci"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let legacy = helpers::ci_mentions(ctx, &["semgrep", "codeql", "sonar"]);
        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::Testing,
            crate::packs::MapsTo::StaticAnalysisCi,
            legacy,
            "No static analysis steps detected in CI workflows.",
            "Static analysis tooling configuration detected.",
            "Static analysis steps detected in CI/scripts.",
            "Run linters/static analysis in CI (credo, clippy, eslint, sobelow, etc.).",
        )
    }
}

struct TypeCheckingCi;
impl Rule for TypeCheckingCi {
    fn id(&self) -> &'static str {
        "testing.type_checking_ci"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::Testing,
            crate::packs::MapsTo::TypeCheckingCi,
            Vec::new(),
            "No type-checking steps detected in CI workflows.",
            "Type-checking configuration detected.",
            "Type-checking / compile checks detected in CI/scripts.",
            "Add tsc/mypy/cargo check/mix compile --warnings-as-errors (or equivalent) to CI.",
        )
    }
}

struct PropertyTesting;
impl Rule for PropertyTesting {
    fn id(&self) -> &'static str {
        "testing.property"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut legacy = Vec::new();
        for needle in [
            "proptest",
            "hypothesis",
            "quickcheck",
            "stream_data",
            "streamdata",
            "proper",
            "fscheck",
            "jqwik",
            "fast-check",
            "fastcheck",
        ] {
            if let Some(pkg) = ctx.read_text("Cargo.toml") {
                if pkg.to_ascii_lowercase().contains(needle) {
                    legacy.push(EvidenceItem::path("Cargo.toml"));
                }
            }
            if let Some(pkg) = ctx.read_text("package.json") {
                if pkg.to_ascii_lowercase().contains(needle) {
                    legacy.push(EvidenceItem::path("package.json"));
                }
            }
            if let Some(pkg) = ctx.read_text("pyproject.toml") {
                if pkg.to_ascii_lowercase().contains(needle) {
                    legacy.push(EvidenceItem::path("pyproject.toml"));
                }
            }
            if let Some(pkg) = ctx.read_text("mix.exs") {
                if pkg.to_ascii_lowercase().contains(needle) {
                    legacy.push(EvidenceItem::path("mix.exs"));
                }
            }
            if let Some(pkg) = ctx.read_text("go.mod") {
                if pkg.to_ascii_lowercase().contains(needle) {
                    legacy.push(EvidenceItem::path("go.mod"));
                }
            }
        }
        let prop_files = ctx.inventory.find_matching(|p| {
            let l = p.to_ascii_lowercase();
            l.contains("prop_")
                || l.contains("_property_test.")
                || l.contains("property_test")
                || l.contains("/properties/")
                || l.ends_with("_props.rs")
                || l.contains("hypothesis")
        });
        for f in prop_files.into_iter().take(10) {
            legacy.push(EvidenceItem::path(f.relative.clone()));
        }
        legacy.sort_by(|a, b| a.path.cmp(&b.path));
        legacy.dedup();

        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::Testing,
            crate::packs::MapsTo::PropertyTesting,
            legacy,
            "No property-based testing signals detected.",
            "Property-based testing configuration or files detected.",
            "Property-based testing gated in CI/scripts.",
            "Add property tests (proptest, Hypothesis, QuickCheck, StreamData, PropEr, fast-check) and run them in CI.",
        )
    }
}

struct IntegrationTesting;
impl Rule for IntegrationTesting {
    fn id(&self) -> &'static str {
        "testing.integration"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        let mut legacy = Vec::new();
        for d in [
            "tests/integration",
            "test/integration",
            "integration_tests",
            "tests/e2e",
        ] {
            if ctx.has_dir(d) {
                legacy.push(EvidenceItem::path(format!("{d}/")));
            }
        }
        let integ_files = ctx.inventory.find_matching(|p| {
            let l = p.to_ascii_lowercase();
            l.contains("_integration_test.")
                || l.contains("integration_test")
                || l.contains("/integration/")
                || l.contains("testcontainers")
                || l.contains("cucumber")
        });
        for f in integ_files.into_iter().take(10) {
            legacy.push(EvidenceItem::path(f.relative.clone()));
        }
        for path in &ctx.detect_signals().ci_workflow_paths {
            if let Some(text) = ctx.read_text(path) {
                let l = text.to_ascii_lowercase();
                if l.contains("integration")
                    || l.contains("testcontainers")
                    || l.contains("cucumber")
                {
                    legacy.push(EvidenceItem::path_detail(
                        path.clone(),
                        "mentions:integration",
                    ));
                }
            }
        }
        legacy.sort_by(|a, b| a.path.cmp(&b.path));
        legacy.dedup();

        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::Testing,
            crate::packs::MapsTo::IntegrationTesting,
            legacy,
            "No integration testing signals detected.",
            "Integration testing configuration or directories detected.",
            "Integration testing gated in CI/scripts.",
            "Add integration tests (tests/integration, Testcontainers, Cucumber) and run them in CI.",
        )
    }
}

struct UiTesting;
impl Rule for UiTesting {
    fn id(&self) -> &'static str {
        "testing.ui"
    }
    fn evaluate(&self, ctx: &RepoContext) -> Finding {
        if !ui_ecosystem_present(ctx) {
            return Finding::builder(self.id(), Category::Testing)
                .status(Status::NotApplicable)
                .confidence(Confidence::High)
                .summary("No UI/web ecosystem detected; UI testing not applicable.")
                .build();
        }

        let mut legacy = Vec::new();
        for name in [
            "playwright.config.ts",
            "playwright.config.js",
            "cypress.config.ts",
            "cypress.config.js",
            "cypress.json",
            "wdio.conf.js",
            "nightwatch.conf.js",
        ] {
            if ctx.has_file(name) {
                legacy.push(EvidenceItem::path(name));
            }
        }
        if let Some(pkg) = ctx.read_text("package.json") {
            let l = pkg.to_ascii_lowercase();
            for needle in [
                "playwright",
                "cypress",
                "selenium",
                "@testing-library",
                "vitest",
                "puppeteer",
                "webdriverio",
            ] {
                if l.contains(needle) {
                    legacy.push(EvidenceItem::path_detail(
                        "package.json",
                        format!("mentions:{needle}"),
                    ));
                }
            }
        }

        helpers::finding_from_pack_or_legacy(
            ctx,
            self.id(),
            Category::Testing,
            crate::packs::MapsTo::UiTesting,
            legacy,
            "UI ecosystem present but no UI/E2E test signals detected.",
            "UI/E2E testing configuration detected.",
            "UI/E2E testing gated in CI/scripts.",
            "Add Playwright, Cypress, Selenium, or browser Vitest tests and gate them in CI.",
        )
    }
}

fn ui_ecosystem_present(ctx: &RepoContext) -> bool {
    let tooling = crate::packs::for_repo(ctx);
    for id in [
        "playwright",
        "cypress",
        "nextjs",
        "react",
        "vue",
        "svelte",
        "angular",
        "javascript",
        "typescript",
    ] {
        if tooling.active_packs.iter().any(|p| p == id) {
            // JS/TS alone is weak; require stronger UI signal below unless playwright/cypress/framework
            if matches!(
                id,
                "playwright" | "cypress" | "nextjs" | "react" | "vue" | "svelte" | "angular"
            ) {
                return true;
            }
        }
    }
    if ctx.has_file("playwright.config.ts")
        || ctx.has_file("playwright.config.js")
        || ctx.has_file("cypress.config.ts")
        || ctx.has_file("cypress.config.js")
        || ctx.has_file("cypress.json")
    {
        return true;
    }
    if let Some(pkg) = ctx.read_text("package.json") {
        let l = pkg.to_ascii_lowercase();
        if l.contains("\"react\"")
            || l.contains("\"vue\"")
            || l.contains("\"svelte\"")
            || l.contains("\"@angular/core\"")
            || l.contains("\"next\"")
            || l.contains("playwright")
            || l.contains("cypress")
        {
            return true;
        }
    }
    // HTML-ish frontends
    ctx.inventory
        .find_matching(|p| {
            let l = p.to_ascii_lowercase();
            l.ends_with(".tsx")
                || l.ends_with(".jsx")
                || l.ends_with(".vue")
                || l.ends_with(".svelte")
        })
        .into_iter()
        .next()
        .is_some()
}
