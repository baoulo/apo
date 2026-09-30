//! Built-in language and web ecosystem pack definitions.

use crate::packs::MapsTo;

/// Static pack definition.
#[derive(Debug, Clone)]
pub struct PackDef {
    pub id: &'static str,
    pub kind: super::PackKind,
    /// Exact relative paths that activate the pack.
    pub manifests: &'static [&'static str],
    /// Substring path matches (e.g. `.sql`, `migrations`).
    pub path_contains: &'static [&'static str],
    /// Any inventory basename match activates the pack.
    pub basename_any: &'static [&'static str],
    /// Activate when `package.json` contains these substrings (deps/scripts).
    pub package_json_contains: &'static [&'static str],
    pub tooling: &'static [ToolingDef],
}

#[derive(Debug, Clone)]
pub struct ToolingDef {
    pub id: &'static str,
    pub maps_to: MapsTo,
    pub configs: &'static [&'static str],
    pub ci_commands: &'static [&'static str],
}

macro_rules! tool {
    ($id:literal, $maps:expr, configs: [$($c:literal),* $(,)?], ci: [$($ci:literal),* $(,)?]) => {
        ToolingDef {
            id: $id,
            maps_to: $maps,
            configs: &[$($c),*],
            ci_commands: &[$($ci),*],
        }
    };
}

/// Human-readable pack catalog (id + kind).
pub fn catalog_ids() -> Vec<(&'static str, &'static str)> {
    builtin_packs()
        .iter()
        .map(|p| {
            (
                p.id,
                match p.kind {
                    super::PackKind::Language => "language",
                    super::PackKind::Web => "web",
                },
            )
        })
        .collect()
}

/// All built-in packs (languages + web).
pub fn builtin_packs() -> &'static [PackDef] {
    PACKS
}

static PACKS: &[PackDef] = &[
    // ——— Tier A languages ———
    PackDef {
        id: "rust",
        kind: super::PackKind::Language,
        manifests: &["Cargo.toml"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: ["rustfmt.toml", ".rustfmt.toml"], ci: ["cargo fmt", "rustfmt"]),
            tool!("lint", MapsTo::Linter, configs: ["clippy.toml", ".clippy.toml"], ci: ["cargo clippy", "clippy"]),
            tool!("types", MapsTo::TypeChecker, configs: ["Cargo.toml"], ci: ["cargo check", "cargo build"]),
            tool!("audit", MapsTo::DependencyScanning, configs: ["deny.toml", "audit.toml"], ci: ["cargo audit", "cargo deny"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["cargo test"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["cargo clippy", "cargo deny"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["cargo check", "cargo build"]),
        ],
    },
    PackDef {
        id: "javascript",
        kind: super::PackKind::Language,
        manifests: &["package.json"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".prettierrc", ".prettierrc.js", ".prettierrc.cjs", ".prettierrc.json", ".prettierrc.yaml", ".prettierrc.yml", "prettier.config.js", "prettier.config.cjs"], ci: ["prettier", "npm run format", "pnpm format", "yarn format"]),
            tool!("lint", MapsTo::Linter, configs: [".eslintrc", ".eslintrc.js", ".eslintrc.cjs", ".eslintrc.json", ".eslintrc.yml", "eslint.config.js", "eslint.config.mjs", "eslint.config.cjs"], ci: ["eslint", "npm run lint", "pnpm lint", "yarn lint"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["npm audit", "pnpm audit", "yarn audit", "osv-scanner"]),
            tool!("test", MapsTo::TestFramework, configs: ["jest.config.js", "jest.config.ts", "vitest.config.ts", "vitest.config.js", "karma.conf.js"], ci: ["npm test", "pnpm test", "yarn test", "vitest", "jest"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["eslint"]),
        ],
    },
    PackDef {
        id: "typescript",
        kind: super::PackKind::Language,
        manifests: &["tsconfig.json", "jsconfig.json"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("types", MapsTo::TypeChecker, configs: ["tsconfig.json", "jsconfig.json"], ci: ["tsc", "vue-tsc"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["tsc", "vue-tsc", "typescript"]),
            tool!("lint", MapsTo::Linter, configs: [], ci: ["eslint", "tslint"]),
        ],
    },
    PackDef {
        id: "python",
        kind: super::PackKind::Language,
        manifests: &["pyproject.toml", "setup.py", "requirements.txt", "Pipfile"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: ["pyproject.toml", ".isort.cfg"], ci: ["black", "ruff format", "isort"]),
            tool!("lint", MapsTo::Linter, configs: ["ruff.toml", ".flake8", "pylintrc", ".pylintrc", "setup.cfg", "pyproject.toml"], ci: ["ruff", "flake8", "pylint"]),
            tool!("types", MapsTo::TypeChecker, configs: ["mypy.ini", ".mypy.ini", "pyrightconfig.json", "pyproject.toml"], ci: ["mypy", "pyright", "pytype"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["pip-audit", "safety", "osv-scanner"]),
            tool!("test", MapsTo::TestFramework, configs: ["pytest.ini", "tox.ini", "pyproject.toml"], ci: ["pytest", "unittest", "tox", "nox"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["mypy", "pyright"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["ruff", "flake8", "pylint", "bandit"]),
        ],
    },
    PackDef {
        id: "go",
        kind: super::PackKind::Language,
        manifests: &["go.mod"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [], ci: ["gofmt", "go fmt", "gofumpt"]),
            tool!("lint", MapsTo::Linter, configs: [".golangci.yml", ".golangci.yaml"], ci: ["golangci-lint", "staticcheck", "go vet"]),
            tool!("types", MapsTo::TypeChecker, configs: ["go.mod"], ci: ["go build", "go vet"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["govulncheck", "nancy", "osv-scanner"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["go test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["go build", "go vet"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["golangci-lint", "staticcheck"]),
        ],
    },
    PackDef {
        id: "java",
        kind: super::PackKind::Language,
        manifests: &["pom.xml"],
        path_contains: &[],
        basename_any: &["build.gradle", "build.gradle.kts"],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".editorconfig"], ci: ["spotless", "formatter"]),
            tool!("lint", MapsTo::Linter, configs: ["checkstyle.xml", "pmd.xml", "spotbugs.xml"], ci: ["checkstyle", "pmd", "spotbugs"]),
            tool!("types", MapsTo::TypeChecker, configs: ["pom.xml"], ci: ["mvn compile", "gradle build", "./gradlew"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["owasp", "dependency-check", "snyk"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["mvn test", "gradle test", "./gradlew test"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["checkstyle", "spotbugs", "pmd"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["mvn compile", "gradle build"]),
        ],
    },
    PackDef {
        id: "csharp",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".csproj", ".sln"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".editorconfig", "dotnet-format.json"], ci: ["dotnet format"]),
            tool!("lint", MapsTo::Linter, configs: [".editorconfig", "stylecop.json"], ci: ["dotnet format", "analyzers"]),
            tool!("types", MapsTo::TypeChecker, configs: [], ci: ["dotnet build"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["dotnet list package --vulnerable", "nuget audit"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["dotnet test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["dotnet build"]),
        ],
    },
    PackDef {
        id: "php",
        kind: super::PackKind::Language,
        manifests: &["composer.json"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".php-cs-fixer.php", "phpcs.xml", "phpcs.xml.dist"], ci: ["php-cs-fixer", "phpcbf", "phpcs"]),
            tool!("lint", MapsTo::Linter, configs: ["phpstan.neon", "phpstan.neon.dist", "psalm.xml", "psalm.xml.dist"], ci: ["phpstan", "psalm", "phpcs"]),
            tool!("types", MapsTo::TypeChecker, configs: ["phpstan.neon", "psalm.xml"], ci: ["phpstan", "psalm"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["composer audit", "security-checker"]),
            tool!("test", MapsTo::TestFramework, configs: ["phpunit.xml", "phpunit.xml.dist"], ci: ["phpunit", "pest"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["phpstan", "psalm"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["phpstan", "psalm"]),
        ],
    },
    PackDef {
        id: "ruby",
        kind: super::PackKind::Language,
        manifests: &["Gemfile"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".rubocop.yml", ".standard.yml"], ci: ["rubocop", "standardrb"]),
            tool!("lint", MapsTo::Linter, configs: [".rubocop.yml", ".reek.yml"], ci: ["rubocop", "reek"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["bundler-audit", "brakeman"]),
            tool!("test", MapsTo::TestFramework, configs: [".rspec", "Rakefile"], ci: ["rspec", "rake test", "minitest"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["rubocop", "brakeman"]),
        ],
    },
    PackDef {
        id: "elixir",
        kind: super::PackKind::Language,
        manifests: &["mix.exs"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".formatter.exs"], ci: ["mix format --check-formatted", "mix format"]),
            tool!("lint", MapsTo::Linter, configs: [".credo.exs"], ci: ["mix credo --strict", "mix credo"]),
            tool!("types", MapsTo::TypeChecker, configs: [".dialyzer_ignore.exs", "dialyzer.exs"], ci: ["mix dialyzer"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["mix hex.audit", "mix hex.outdated"]),
            tool!("security", MapsTo::StaticAnalysisCi, configs: [".sobelow-conf", ".sobelow-skips"], ci: ["mix sobelow --config", "mix sobelow"]),
            tool!("compile", MapsTo::TypeCheckingCi, configs: [], ci: ["mix compile --warnings-as-errors", "mix compile"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["mix test"]),
            tool!("doctor", MapsTo::DocTooling, configs: [], ci: ["mix doctor --full", "mix doctor"]),
            tool!("docs", MapsTo::DocTooling, configs: [], ci: ["mix docs --warnings-as-errors", "mix docs"]),
        ],
    },
    // ——— Tier B ———
    PackDef {
        id: "cpp",
        kind: super::PackKind::Language,
        manifests: &[
            "CMakeLists.txt",
            "meson.build",
            "conanfile.txt",
            "conanfile.py",
            "vcpkg.json",
        ],
        path_contains: &[".cxx", ".cpp"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".clang-format"], ci: ["clang-format"]),
            tool!("lint", MapsTo::Linter, configs: [".clang-tidy"], ci: ["clang-tidy", "cppcheck"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["cmake", "ninja", "make"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["ctest", "gtest"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["clang-tidy", "cppcheck"]),
        ],
    },
    PackDef {
        id: "c",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".c"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".clang-format"], ci: ["clang-format"]),
            tool!("lint", MapsTo::Linter, configs: [".clang-tidy"], ci: ["clang-tidy", "cppcheck"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["clang-tidy"]),
        ],
    },
    PackDef {
        id: "kotlin",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".kt", ".kts"],
        basename_any: &["build.gradle.kts"],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".editorconfig"], ci: ["ktlint", "spotless"]),
            tool!("lint", MapsTo::Linter, configs: ["detekt.yml", ".detekt.yml"], ci: ["detekt", "ktlint"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["gradle test", "./gradlew test"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["detekt", "ktlint"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["gradle build", "./gradlew"]),
        ],
    },
    PackDef {
        id: "swift",
        kind: super::PackKind::Language,
        manifests: &["Package.swift"],
        path_contains: &[".xcodeproj", ".xcworkspace"],
        basename_any: &["Podfile"],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".swiftformat"], ci: ["swiftformat", "swift-format"]),
            tool!("lint", MapsTo::Linter, configs: [".swiftlint.yml"], ci: ["swiftlint"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["swift test", "xcodebuild test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["swift build", "xcodebuild"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["swiftlint"]),
        ],
    },
    PackDef {
        id: "objc",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".m", ".mm"],
        basename_any: &["Podfile"],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".clang-format"], ci: ["clang-format"]),
            tool!("lint", MapsTo::Linter, configs: [".clang-tidy"], ci: ["clang-tidy"]),
        ],
    },
    PackDef {
        id: "dart",
        kind: super::PackKind::Language,
        manifests: &["pubspec.yaml"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: ["analysis_options.yaml"], ci: ["dart format", "flutter format"]),
            tool!("lint", MapsTo::Linter, configs: ["analysis_options.yaml"], ci: ["dart analyze", "flutter analyze"]),
            tool!("types", MapsTo::TypeChecker, configs: ["analysis_options.yaml"], ci: ["dart analyze"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["dart test", "flutter test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["dart analyze", "flutter analyze"]),
        ],
    },
    PackDef {
        id: "scala",
        kind: super::PackKind::Language,
        manifests: &["build.sbt"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".scalafmt.conf"], ci: ["scalafmt", "sbt scalafmtCheck"]),
            tool!("lint", MapsTo::Linter, configs: [".scalafix.conf"], ci: ["scalafix", "sbt scalafix"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["sbt test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["sbt compile"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["scalafix"]),
        ],
    },
    PackDef {
        id: "erlang",
        kind: super::PackKind::Language,
        manifests: &["rebar.config", "erlang.mk"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: ["elvis.config"], ci: ["elvis", "rebar3 dialyzer"]),
            tool!("types", MapsTo::TypeChecker, configs: [], ci: ["dialyzer", "rebar3 dialyzer"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["rebar3 eunit", "rebar3 ct"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["dialyzer"]),
        ],
    },
    // ——— Tier C ———
    PackDef {
        id: "haskell",
        kind: super::PackKind::Language,
        manifests: &["package.yaml", "stack.yaml", "cabal.project"],
        path_contains: &[".cabal"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: ["fourmolu.yaml", ".ormolu", "stylish-haskell.yaml"], ci: ["fourmolu", "ormolu", "stylish-haskell"]),
            tool!("lint", MapsTo::Linter, configs: [".hlint.yaml", "hlint.yaml"], ci: ["hlint"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["stack test", "cabal test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["stack build", "cabal build"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["hlint"]),
        ],
    },
    PackDef {
        id: "clojure",
        kind: super::PackKind::Language,
        manifests: &["deps.edn", "project.clj", "bb.edn"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [".clj-kondo/config.edn"], ci: ["clj-kondo", "eastwood"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["clojure -X:test", "lein test", "bb test"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["clj-kondo"]),
        ],
    },
    PackDef {
        id: "r",
        kind: super::PackKind::Language,
        manifests: &["DESCRIPTION", "renv.lock"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [], ci: ["styler"]),
            tool!("lint", MapsTo::Linter, configs: [".lintr"], ci: ["lintr"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["testthat", "R CMD check", "rcmdcheck"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["lintr"]),
        ],
    },
    PackDef {
        id: "lua",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".lua", ".rockspec"],
        basename_any: &["selene.toml", "stylua.toml"],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: ["stylua.toml", ".stylua.toml"], ci: ["stylua"]),
            tool!("lint", MapsTo::Linter, configs: ["selene.toml"], ci: ["selene", "luacheck"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["selene", "luacheck"]),
        ],
    },
    PackDef {
        id: "perl",
        kind: super::PackKind::Language,
        manifests: &["cpanfile", "Makefile.PL", "Build.PL"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [".perlcriticrc"], ci: ["perlcritic"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["prove", "perl Build test"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["perlcritic"]),
        ],
    },
    PackDef {
        id: "shell",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".sh", ".bash"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [], ci: ["shfmt"]),
            tool!("lint", MapsTo::Linter, configs: [".shellcheckrc"], ci: ["shellcheck"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["shellcheck"]),
        ],
    },
    PackDef {
        id: "powershell",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[".ps1", ".psm1"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: ["PSScriptAnalyzerSettings.psd1"], ci: ["PSScriptAnalyzer", "Invoke-ScriptAnalyzer"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["PSScriptAnalyzer"]),
        ],
    },
    PackDef {
        id: "julia",
        kind: super::PackKind::Language,
        manifests: &["Project.toml", "Manifest.toml"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [], ci: ["JET", "Aqua"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["Pkg.test", "julia --project"]),
            tool!("docs", MapsTo::DocTooling, configs: [], ci: ["Documenter", "makedocs"]),
        ],
    },
    PackDef {
        id: "zig",
        kind: super::PackKind::Language,
        manifests: &["build.zig", "build.zig.zon"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [], ci: ["zig fmt"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["zig test", "zig build test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["zig build"]),
        ],
    },
    PackDef {
        id: "nim",
        kind: super::PackKind::Language,
        manifests: &["config.nims"],
        path_contains: &[".nimble"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [], ci: ["nimpretty"]),
            tool!("types", MapsTo::TypeChecker, configs: [], ci: ["nim check"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["testament", "nimble test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["nim check", "nim c"]),
        ],
    },
    PackDef {
        id: "sql",
        kind: super::PackKind::Language,
        manifests: &[],
        path_contains: &[
            ".sql",
            "migrations/",
            "db/migrate",
            "prisma/migrations",
            "supabase/migrations",
            "flyway",
            "liquibase",
        ],
        basename_any: &[".sqlfluff", "sqlfluff"],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [".sqlfluff", "sqlfluff"], ci: ["sqlfmt", "sqlfluff fix"]),
            tool!("lint", MapsTo::Linter, configs: [".sqlfluff"], ci: ["sqlfluff lint", "sqlfluff"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["pg_prove", "pgtap"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["sqlfluff"]),
        ],
    },
    PackDef {
        id: "solidity",
        kind: super::PackKind::Language,
        manifests: &[
            "foundry.toml",
            "hardhat.config.js",
            "hardhat.config.ts",
            "truffle-config.js",
        ],
        path_contains: &[".sol"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [".solhint.json", ".solhintrc"], ci: ["solhint", "slither"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["forge test", "hardhat test", "npx hardhat test"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["slither", "solhint"]),
            tool!("audit", MapsTo::DependencyScanning, configs: [], ci: ["slither"]),
        ],
    },
    PackDef {
        id: "nix",
        kind: super::PackKind::Language,
        manifests: &["flake.nix", "shell.nix", "default.nix"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("format", MapsTo::Formatter, configs: [], ci: ["nix fmt", "alejandra", "nixpkgs-fmt"]),
            tool!("lint", MapsTo::Linter, configs: [], ci: ["deadnix", "statix"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["deadnix", "statix"]),
        ],
    },
    // ——— Web ecosystems (additive) ———
    PackDef {
        id: "react",
        kind: super::PackKind::Web,
        manifests: &[],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &["\"react\""],
        tooling: &[
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["react-scripts test", "@testing-library/react"]),
            tool!("lint", MapsTo::Linter, configs: [], ci: ["eslint-plugin-react"]),
        ],
    },
    PackDef {
        id: "nextjs",
        kind: super::PackKind::Web,
        manifests: &["next.config.js", "next.config.mjs", "next.config.ts"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [], ci: ["next lint"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["next build", "tsc"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["next lint"]),
        ],
    },
    PackDef {
        id: "vue",
        kind: super::PackKind::Web,
        manifests: &[],
        path_contains: &[".vue"],
        basename_any: &[],
        package_json_contains: &["\"vue\""],
        tooling: &[
            tool!("types", MapsTo::TypeChecker, configs: [], ci: ["vue-tsc"]),
            tool!("lint", MapsTo::Linter, configs: [], ci: ["eslint", "vue-tsc"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["vue-tsc"]),
        ],
    },
    PackDef {
        id: "nuxt",
        kind: super::PackKind::Web,
        manifests: &["nuxt.config.ts", "nuxt.config.js", "nuxt.config.mjs"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [], ci: ["nuxi", "nuxt build"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["nuxi typecheck", "nuxt build"]),
        ],
    },
    PackDef {
        id: "svelte",
        kind: super::PackKind::Web,
        manifests: &["svelte.config.js", "svelte.config.ts"],
        path_contains: &[".svelte"],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("types", MapsTo::TypeChecker, configs: [], ci: ["svelte-check"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["svelte-check"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["vitest"]),
        ],
    },
    PackDef {
        id: "angular",
        kind: super::PackKind::Web,
        manifests: &["angular.json"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [], ci: ["ng lint"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["ng test"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["ng build"]),
            tool!("static", MapsTo::StaticAnalysisCi, configs: [], ci: ["ng lint"]),
        ],
    },
    PackDef {
        id: "remix",
        kind: super::PackKind::Web,
        manifests: &["remix.config.js", "remix.config.ts"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &["@remix-run/"],
        tooling: &[
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["remix vite:build", "tsc"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["vitest", "playwright"]),
        ],
    },
    PackDef {
        id: "astro",
        kind: super::PackKind::Web,
        manifests: &["astro.config.mjs", "astro.config.ts", "astro.config.js"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("types", MapsTo::TypeChecker, configs: [], ci: ["astro check"]),
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["astro check", "astro build"]),
        ],
    },
    PackDef {
        id: "vite",
        kind: super::PackKind::Web,
        manifests: &["vite.config.ts", "vite.config.js", "vite.config.mjs"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("types_ci", MapsTo::TypeCheckingCi, configs: [], ci: ["vite build", "tsc"]),
            tool!("test", MapsTo::TestFramework, configs: [], ci: ["vitest"]),
        ],
    },
    PackDef {
        id: "css_tooling",
        kind: super::PackKind::Web,
        manifests: &[
            "tailwind.config.js",
            "tailwind.config.ts",
            "postcss.config.js",
            "postcss.config.cjs",
        ],
        path_contains: &[],
        basename_any: &[".stylelintrc", "stylelint.config.js"],
        package_json_contains: &[],
        tooling: &[
            tool!("lint", MapsTo::Linter, configs: [".stylelintrc", ".stylelintrc.json", "stylelint.config.js"], ci: ["stylelint"]),
            tool!("format", MapsTo::Formatter, configs: ["prettier.config.js"], ci: ["prettier"]),
        ],
    },
    PackDef {
        id: "playwright",
        kind: super::PackKind::Web,
        manifests: &["playwright.config.ts", "playwright.config.js"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("e2e", MapsTo::TestFramework, configs: ["playwright.config.ts", "playwright.config.js"], ci: ["playwright test", "npx playwright"]),
        ],
    },
    PackDef {
        id: "cypress",
        kind: super::PackKind::Web,
        manifests: &["cypress.config.ts", "cypress.config.js", "cypress.json"],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &[],
        tooling: &[
            tool!("e2e", MapsTo::TestFramework, configs: ["cypress.config.ts", "cypress.config.js"], ci: ["cypress run", "cypress open"]),
        ],
    },
    PackDef {
        id: "a11y",
        kind: super::PackKind::Web,
        manifests: &[],
        path_contains: &[],
        basename_any: &[],
        package_json_contains: &["axe-core", "pa11y", "lighthouse"],
        tooling: &[
            tool!("axe", MapsTo::StaticAnalysisCi, configs: [], ci: ["axe", "pa11y", "lighthouse-ci", "@axe-core"]),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_core_packs() {
        let ids: Vec<_> = builtin_packs().iter().map(|p| p.id).collect();
        for need in [
            "rust",
            "elixir",
            "javascript",
            "typescript",
            "python",
            "go",
            "nim",
            "sql",
            "nextjs",
            "playwright",
            "nix",
            "solidity",
        ] {
            assert!(ids.contains(&need), "missing pack {need}");
        }
        assert!(builtin_packs().len() >= 30);
    }
}
