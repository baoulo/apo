# Language and web packs

Observational **ecosystem packs** teach hygiene rules which configs and CI/script
needles to look for. Packs never execute toolchains; they only observe files and text.

There are **47** built-in packs (34 language + 13 web).
Force-enable or overlay via [`.apo.toml`](../examples/apo.toml) (`[ecosystem]` / `[[tooling]]`).

Source of truth: [`src/packs/catalog.rs`](../src/packs/catalog.rs).

## How packs work

1. **Activation** — manifests, path substrings, basenames, and/or `package.json` needles.
2. **Contribution** — each tooling entry maps to a hygiene rule id (`MapsTo`).
3. **Evidence** — present configs → Present; CI/script mentions → often Enforced.

Tables below are sorted by pack id; tooling rows by tooling id.

## Index

| Pack id | Kind | Activates when |
|---------|------|----------------|
| `a11y` | Web | `package.json` `axe-core`, `pa11y`, `lighthouse` |
| `angular` | Web | `angular.json` |
| `astro` | Web | `astro.config.mjs`, `astro.config.ts`, `astro.config.js` |
| `c` | Language | paths `.c` |
| `clojure` | Language | `deps.edn`, `project.clj`, `bb.edn` |
| `cobol` | Language | paths `.cob`, `.cbl` |
| `cpp` | Language | `CMakeLists.txt`, `meson.build`, `conanfile.txt`, `conanfile.py`, …; paths `.cxx`, `.cpp` |
| `csharp` | Language | paths `.csproj`, `.sln` |
| `css_tooling` | Web | `tailwind.config.js`, `tailwind.config.ts`, `postcss.config.js`, `postcss.config.cjs`; basenames `.stylelintrc`, `stylelint.config.js` |
| `cypress` | Web | `cypress.config.ts`, `cypress.config.js`, `cypress.json` |
| `dart` | Language | `pubspec.yaml` |
| `elixir` | Language | `mix.exs` |
| `erlang` | Language | `rebar.config`, `erlang.mk` |
| `fortran` | Language | `fpm.toml`; paths `.f90`, `.f95`, `.f03`, `.f08` |
| `go` | Language | `go.mod` |
| `haskell` | Language | `package.yaml`, `stack.yaml`, `cabal.project`; paths `.cabal` |
| `java` | Language | `pom.xml`; basenames `build.gradle`, `build.gradle.kts` |
| `javascript` | Language | `package.json` |
| `julia` | Language | `Project.toml`, `Manifest.toml` |
| `kotlin` | Language | paths `.kt`, `.kts`; basenames `build.gradle.kts` |
| `lua` | Language | paths `.lua`, `.rockspec`; basenames `selene.toml`, `stylua.toml` |
| `nextjs` | Web | `next.config.js`, `next.config.mjs`, `next.config.ts` |
| `nim` | Language | `config.nims`; paths `.nimble` |
| `nix` | Language | `flake.nix`, `shell.nix`, `default.nix` |
| `nuxt` | Web | `nuxt.config.ts`, `nuxt.config.js`, `nuxt.config.mjs` |
| `objc` | Language | paths `.m`, `.mm`; basenames `Podfile` |
| `pascal` | Language | paths `.pas`, `.lpr`, `.dpr`, `.lpi` |
| `perl` | Language | `cpanfile`, `Makefile.PL`, `Build.PL` |
| `php` | Language | `composer.json` |
| `playwright` | Web | `playwright.config.ts`, `playwright.config.js` |
| `powershell` | Language | paths `.ps1`, `.psm1` |
| `python` | Language | `pyproject.toml`, `setup.py`, `requirements.txt`, `Pipfile` |
| `r` | Language | `DESCRIPTION`, `renv.lock` |
| `react` | Web | `package.json` `"react"` |
| `remix` | Web | `remix.config.js`, `remix.config.ts`; `package.json` `@remix-run/` |
| `ruby` | Language | `Gemfile` |
| `rust` | Language | `Cargo.toml` |
| `scala` | Language | `build.sbt` |
| `shell` | Language | paths `.sh`, `.bash` |
| `solidity` | Language | `foundry.toml`, `hardhat.config.js`, `hardhat.config.ts`, `truffle-config.js`; paths `.sol` |
| `sql` | Language | paths `.sql`, `migrations/`, `db/migrate`, `prisma/migrations`, …; basenames `.sqlfluff`, `sqlfluff` |
| `svelte` | Web | `svelte.config.js`, `svelte.config.ts`; paths `.svelte` |
| `swift` | Language | `Package.swift`; paths `.xcodeproj`, `.xcworkspace`; basenames `Podfile` |
| `typescript` | Language | `tsconfig.json`, `jsconfig.json` |
| `vite` | Web | `vite.config.ts`, `vite.config.js`, `vite.config.mjs` |
| `vue` | Web | paths `.vue`; `package.json` `"vue"` |
| `zig` | Language | `build.zig`, `build.zig.zon` |

## Language packs

### `c`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.c` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.clang-format` | `clang-format` |
| `lint` | `local_development.linter` | `.clang-tidy` | `clang-tidy`, `cppcheck` |
| `static` | `testing.static_analysis_ci` | — | `clang-tidy` |

### `clojure`

| Signal | Values |
|--------|--------|
| Manifests | `deps.edn`, `project.clj`, `bb.edn` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | `.clj-kondo/config.edn` | `clj-kondo`, `eastwood` |
| `static` | `testing.static_analysis_ci` | — | `clj-kondo` |
| `test` | `testing.framework` | — | `clojure -X:test`, `lein test`, `bb test` |

### `cobol`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.cob`, `.cbl` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `cobol-format`, `cobc` |
| `lint` | `local_development.linter` | — | `cobolci`, `gnucobol`, `cobc` |
| `test` | `testing.framework` | — | `cobolci`, `cobc`, `make test` |
| `types_ci` | `testing.type_checking_ci` | — | `cobc`, `cobolci` |

### `cpp`

| Signal | Values |
|--------|--------|
| Manifests | `CMakeLists.txt`, `meson.build`, `conanfile.txt`, `conanfile.py`, `vcpkg.json` |
| Path contains | `.cxx`, `.cpp` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.clang-format` | `clang-format` |
| `lint` | `local_development.linter` | `.clang-tidy` | `clang-tidy`, `cppcheck` |
| `static` | `testing.static_analysis_ci` | — | `clang-tidy`, `cppcheck` |
| `test` | `testing.framework` | — | `ctest`, `gtest` |
| `types_ci` | `testing.type_checking_ci` | — | `cmake`, `ninja`, `make` |

### `csharp`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.csproj`, `.sln` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `dotnet list package --vulnerable`, `nuget audit` |
| `format` | `local_development.formatter` | `.editorconfig`, `dotnet-format.json` | `dotnet format` |
| `lint` | `local_development.linter` | `.editorconfig`, `stylecop.json` | `dotnet format`, `analyzers` |
| `test` | `testing.framework` | — | `dotnet test` |
| `types` | `local_development.type_checker` | — | `dotnet build` |
| `types_ci` | `testing.type_checking_ci` | — | `dotnet build` |

### `dart`

| Signal | Values |
|--------|--------|
| Manifests | `pubspec.yaml` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `analysis_options.yaml` | `dart format`, `flutter format` |
| `lint` | `local_development.linter` | `analysis_options.yaml` | `dart analyze`, `flutter analyze` |
| `test` | `testing.framework` | — | `dart test`, `flutter test` |
| `types` | `local_development.type_checker` | `analysis_options.yaml` | `dart analyze` |
| `types_ci` | `testing.type_checking_ci` | — | `dart analyze`, `flutter analyze` |

### `elixir`

| Signal | Values |
|--------|--------|
| Manifests | `mix.exs` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `mix hex.audit`, `mix hex.outdated` |
| `compile` | `testing.type_checking_ci` | — | `mix compile --warnings-as-errors`, `mix compile` |
| `docs` | `documentation.doc_tooling` | — | `mix docs --warnings-as-errors`, `mix docs` |
| `doctor` | `documentation.doc_tooling` | — | `mix doctor --full`, `mix doctor` |
| `format` | `local_development.formatter` | `.formatter.exs` | `mix format --check-formatted`, `mix format` |
| `integration` | `testing.integration` | — | `integration` |
| `lint` | `local_development.linter` | `.credo.exs` | `mix credo --strict`, `mix credo` |
| `property` | `testing.property` | — | `stream_data`, `propcheck` |
| `security` | `testing.static_analysis_ci` | `.sobelow-conf`, `.sobelow-skips` | `mix sobelow --config`, `mix sobelow` |
| `test` | `testing.framework` | — | `mix test` |
| `types` | `local_development.type_checker` | `.dialyzer_ignore.exs`, `dialyzer.exs` | `mix dialyzer` |

### `erlang`

| Signal | Values |
|--------|--------|
| Manifests | `rebar.config`, `erlang.mk` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | `elvis.config` | `elvis`, `rebar3 dialyzer` |
| `test` | `testing.framework` | — | `rebar3 eunit`, `rebar3 ct` |
| `types` | `local_development.type_checker` | — | `dialyzer`, `rebar3 dialyzer` |
| `types_ci` | `testing.type_checking_ci` | — | `dialyzer` |

### `fortran`

| Signal | Values |
|--------|--------|
| Manifests | `fpm.toml` |
| Path contains | `.f90`, `.f95`, `.f03`, `.f08` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `fprettify`, `findent` |
| `lint` | `local_development.linter` | — | `fortitude`, `flinter`, `fortran-linter` |
| `static` | `testing.static_analysis_ci` | — | `fortitude`, `flinter` |
| `test` | `testing.framework` | `fpm.toml` | `fpm test` |
| `types_ci` | `testing.type_checking_ci` | — | `fpm build`, `cmake` |

### `go`

| Signal | Values |
|--------|--------|
| Manifests | `go.mod` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `govulncheck`, `nancy`, `osv-scanner` |
| `format` | `local_development.formatter` | — | `gofmt`, `go fmt`, `gofumpt` |
| `lint` | `local_development.linter` | `.golangci.yml`, `.golangci.yaml` | `golangci-lint`, `staticcheck`, `go vet` |
| `static` | `testing.static_analysis_ci` | — | `golangci-lint`, `staticcheck` |
| `test` | `testing.framework` | — | `go test` |
| `types` | `local_development.type_checker` | `go.mod` | `go build`, `go vet` |
| `types_ci` | `testing.type_checking_ci` | — | `go build`, `go vet` |

### `haskell`

| Signal | Values |
|--------|--------|
| Manifests | `package.yaml`, `stack.yaml`, `cabal.project` |
| Path contains | `.cabal` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `fourmolu.yaml`, `.ormolu`, `stylish-haskell.yaml` | `fourmolu`, `ormolu`, `stylish-haskell` |
| `lint` | `local_development.linter` | `.hlint.yaml`, `hlint.yaml` | `hlint` |
| `static` | `testing.static_analysis_ci` | — | `hlint` |
| `test` | `testing.framework` | — | `stack test`, `cabal test` |
| `types_ci` | `testing.type_checking_ci` | — | `stack build`, `cabal build` |

### `java`

| Signal | Values |
|--------|--------|
| Manifests | `pom.xml` |
| Path contains | — |
| Basename any | `build.gradle`, `build.gradle.kts` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `owasp`, `dependency-check`, `snyk` |
| `format` | `local_development.formatter` | `.editorconfig` | `spotless`, `formatter` |
| `lint` | `local_development.linter` | `checkstyle.xml`, `pmd.xml`, `spotbugs.xml` | `checkstyle`, `pmd`, `spotbugs` |
| `static` | `testing.static_analysis_ci` | — | `checkstyle`, `spotbugs`, `pmd` |
| `test` | `testing.framework` | — | `mvn test`, `gradle test`, `./gradlew test` |
| `types` | `local_development.type_checker` | `pom.xml` | `mvn compile`, `gradle build`, `./gradlew` |
| `types_ci` | `testing.type_checking_ci` | — | `mvn compile`, `gradle build` |

### `javascript`

| Signal | Values |
|--------|--------|
| Manifests | `package.json` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `npm audit`, `pnpm audit`, `yarn audit`, `osv-scanner` |
| `format` | `local_development.formatter` | `.prettierrc`, `.prettierrc.js`, `.prettierrc.cjs`, `.prettierrc.json`, `.prettierrc.yaml` | `prettier`, `npm run format`, `pnpm format`, `yarn format` |
| `lint` | `local_development.linter` | `.eslintrc`, `.eslintrc.js`, `.eslintrc.cjs`, `.eslintrc.json`, `.eslintrc.yml` | `eslint`, `npm run lint`, `pnpm lint`, `yarn lint` |
| `static` | `testing.static_analysis_ci` | — | `eslint` |
| `test` | `testing.framework` | `jest.config.js`, `jest.config.ts`, `vitest.config.ts`, `vitest.config.js`, `karma.conf.js` | `npm test`, `pnpm test`, `yarn test`, `vitest`, `jest` |

### `julia`

| Signal | Values |
|--------|--------|
| Manifests | `Project.toml`, `Manifest.toml` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `docs` | `documentation.doc_tooling` | — | `Documenter`, `makedocs` |
| `lint` | `local_development.linter` | — | `JET`, `Aqua` |
| `test` | `testing.framework` | — | `Pkg.test`, `julia --project` |

### `kotlin`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.kt`, `.kts` |
| Basename any | `build.gradle.kts` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.editorconfig` | `ktlint`, `spotless` |
| `lint` | `local_development.linter` | `detekt.yml`, `.detekt.yml` | `detekt`, `ktlint` |
| `static` | `testing.static_analysis_ci` | — | `detekt`, `ktlint` |
| `test` | `testing.framework` | — | `gradle test`, `./gradlew test` |
| `types_ci` | `testing.type_checking_ci` | — | `gradle build`, `./gradlew` |

### `lua`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.lua`, `.rockspec` |
| Basename any | `selene.toml`, `stylua.toml` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `stylua.toml`, `.stylua.toml` | `stylua` |
| `lint` | `local_development.linter` | `selene.toml` | `selene`, `luacheck` |
| `static` | `testing.static_analysis_ci` | — | `selene`, `luacheck` |

### `nim`

| Signal | Values |
|--------|--------|
| Manifests | `config.nims` |
| Path contains | `.nimble` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `nimpretty` |
| `test` | `testing.framework` | — | `testament`, `nimble test` |
| `types` | `local_development.type_checker` | — | `nim check` |
| `types_ci` | `testing.type_checking_ci` | — | `nim check`, `nim c` |

### `nix`

| Signal | Values |
|--------|--------|
| Manifests | `flake.nix`, `shell.nix`, `default.nix` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `nix fmt`, `alejandra`, `nixpkgs-fmt` |
| `lint` | `local_development.linter` | — | `deadnix`, `statix` |
| `static` | `testing.static_analysis_ci` | — | `deadnix`, `statix` |

### `objc`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.m`, `.mm` |
| Basename any | `Podfile` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.clang-format` | `clang-format` |
| `lint` | `local_development.linter` | `.clang-tidy` | `clang-tidy` |

### `pascal`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.pas`, `.lpr`, `.dpr`, `.lpi` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `ptop`, `jedi` |
| `lint` | `local_development.linter` | — | `fpc`, `pasdoc` |
| `test` | `testing.framework` | — | `fpc`, `lazbuild`, `make test` |
| `types_ci` | `testing.type_checking_ci` | — | `fpc`, `lazbuild` |

### `perl`

| Signal | Values |
|--------|--------|
| Manifests | `cpanfile`, `Makefile.PL`, `Build.PL` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | `.perlcriticrc` | `perlcritic` |
| `static` | `testing.static_analysis_ci` | — | `perlcritic` |
| `test` | `testing.framework` | — | `prove`, `perl Build test` |

### `php`

| Signal | Values |
|--------|--------|
| Manifests | `composer.json` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `composer audit`, `security-checker` |
| `format` | `local_development.formatter` | `.php-cs-fixer.php`, `phpcs.xml`, `phpcs.xml.dist` | `php-cs-fixer`, `phpcbf`, `phpcs` |
| `lint` | `local_development.linter` | `phpstan.neon`, `phpstan.neon.dist`, `psalm.xml`, `psalm.xml.dist` | `phpstan`, `psalm`, `phpcs` |
| `static` | `testing.static_analysis_ci` | — | `phpstan`, `psalm` |
| `test` | `testing.framework` | `phpunit.xml`, `phpunit.xml.dist` | `phpunit`, `pest` |
| `types` | `local_development.type_checker` | `phpstan.neon`, `psalm.xml` | `phpstan`, `psalm` |
| `types_ci` | `testing.type_checking_ci` | — | `phpstan`, `psalm` |

### `powershell`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.ps1`, `.psm1` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | `PSScriptAnalyzerSettings.psd1` | `PSScriptAnalyzer`, `Invoke-ScriptAnalyzer` |
| `static` | `testing.static_analysis_ci` | — | `PSScriptAnalyzer` |

### `python`

| Signal | Values |
|--------|--------|
| Manifests | `pyproject.toml`, `setup.py`, `requirements.txt`, `Pipfile` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `pip-audit`, `safety`, `osv-scanner` |
| `format` | `local_development.formatter` | `pyproject.toml`, `.isort.cfg` | `black`, `ruff format`, `isort` |
| `integration` | `testing.integration` | — | `integration`, `testcontainers` |
| `lint` | `local_development.linter` | `ruff.toml`, `.flake8`, `pylintrc`, `.pylintrc`, `setup.cfg` | `ruff`, `flake8`, `pylint` |
| `property` | `testing.property` | `pyproject.toml` | `hypothesis`, `pytest` |
| `static` | `testing.static_analysis_ci` | — | `ruff`, `flake8`, `pylint`, `bandit` |
| `test` | `testing.framework` | `pytest.ini`, `tox.ini`, `pyproject.toml` | `pytest`, `unittest`, `tox`, `nox` |
| `types` | `local_development.type_checker` | `mypy.ini`, `.mypy.ini`, `pyrightconfig.json`, `pyproject.toml` | `mypy`, `pyright`, `pytype` |
| `types_ci` | `testing.type_checking_ci` | — | `mypy`, `pyright` |

### `r`

| Signal | Values |
|--------|--------|
| Manifests | `DESCRIPTION`, `renv.lock` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `styler` |
| `lint` | `local_development.linter` | `.lintr` | `lintr` |
| `static` | `testing.static_analysis_ci` | — | `lintr` |
| `test` | `testing.framework` | — | `testthat`, `R CMD check`, `rcmdcheck` |

### `ruby`

| Signal | Values |
|--------|--------|
| Manifests | `Gemfile` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `bundler-audit`, `brakeman` |
| `format` | `local_development.formatter` | `.rubocop.yml`, `.standard.yml` | `rubocop`, `standardrb` |
| `lint` | `local_development.linter` | `.rubocop.yml`, `.reek.yml` | `rubocop`, `reek` |
| `static` | `testing.static_analysis_ci` | — | `rubocop`, `brakeman` |
| `test` | `testing.framework` | `.rspec`, `Rakefile` | `rspec`, `rake test`, `minitest` |

### `rust`

| Signal | Values |
|--------|--------|
| Manifests | `Cargo.toml` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | `deny.toml`, `audit.toml` | `cargo audit`, `cargo deny` |
| `docs` | `documentation.doc_tooling` | — | `cargo doc` |
| `format` | `local_development.formatter` | `rustfmt.toml`, `.rustfmt.toml` | `cargo fmt`, `rustfmt` |
| `integration` | `testing.integration` | — | `integration` |
| `lint` | `local_development.linter` | `clippy.toml`, `.clippy.toml` | `cargo clippy`, `clippy` |
| `property` | `testing.property` | — | `proptest`, `cargo test` |
| `static` | `testing.static_analysis_ci` | — | `cargo clippy`, `cargo deny` |
| `test` | `testing.framework` | — | `cargo test` |
| `types` | `local_development.type_checker` | `Cargo.toml` | `cargo check`, `cargo build` |
| `types_ci` | `testing.type_checking_ci` | — | `cargo check`, `cargo build` |

### `scala`

| Signal | Values |
|--------|--------|
| Manifests | `build.sbt` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.scalafmt.conf` | `scalafmt`, `sbt scalafmtCheck` |
| `lint` | `local_development.linter` | `.scalafix.conf` | `scalafix`, `sbt scalafix` |
| `static` | `testing.static_analysis_ci` | — | `scalafix` |
| `test` | `testing.framework` | — | `sbt test` |
| `types_ci` | `testing.type_checking_ci` | — | `sbt compile` |

### `shell`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.sh`, `.bash` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `shfmt` |
| `lint` | `local_development.linter` | `.shellcheckrc` | `shellcheck` |
| `static` | `testing.static_analysis_ci` | — | `shellcheck` |

### `solidity`

| Signal | Values |
|--------|--------|
| Manifests | `foundry.toml`, `hardhat.config.js`, `hardhat.config.ts`, `truffle-config.js` |
| Path contains | `.sol` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `audit` | `security.dependency_scanning` | — | `slither` |
| `lint` | `local_development.linter` | `.solhint.json`, `.solhintrc` | `solhint`, `slither` |
| `static` | `testing.static_analysis_ci` | — | `slither`, `solhint` |
| `test` | `testing.framework` | — | `forge test`, `hardhat test`, `npx hardhat test` |

### `sql`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.sql`, `migrations/`, `db/migrate`, `prisma/migrations`, `supabase/migrations`, `flyway`, `liquibase` |
| Basename any | `.sqlfluff`, `sqlfluff` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.sqlfluff`, `sqlfluff` | `sqlfmt`, `sqlfluff fix` |
| `lint` | `local_development.linter` | `.sqlfluff` | `sqlfluff lint`, `sqlfluff` |
| `static` | `testing.static_analysis_ci` | — | `sqlfluff` |
| `test` | `testing.framework` | — | `pg_prove`, `pgtap` |

### `swift`

| Signal | Values |
|--------|--------|
| Manifests | `Package.swift` |
| Path contains | `.xcodeproj`, `.xcworkspace` |
| Basename any | `Podfile` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `.swiftformat` | `swiftformat`, `swift-format` |
| `lint` | `local_development.linter` | `.swiftlint.yml` | `swiftlint` |
| `static` | `testing.static_analysis_ci` | — | `swiftlint` |
| `test` | `testing.framework` | — | `swift test`, `xcodebuild test` |
| `types_ci` | `testing.type_checking_ci` | — | `swift build`, `xcodebuild` |

### `typescript`

| Signal | Values |
|--------|--------|
| Manifests | `tsconfig.json`, `jsconfig.json` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | — | `eslint`, `tslint` |
| `types` | `local_development.type_checker` | `tsconfig.json`, `jsconfig.json` | `tsc`, `vue-tsc` |
| `types_ci` | `testing.type_checking_ci` | — | `tsc`, `vue-tsc`, `typescript` |

### `zig`

| Signal | Values |
|--------|--------|
| Manifests | `build.zig`, `build.zig.zon` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | — | `zig fmt` |
| `test` | `testing.framework` | — | `zig test`, `zig build test` |
| `types_ci` | `testing.type_checking_ci` | — | `zig build` |

## Web packs

### `a11y`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | — |
| Basename any | — |
| `package.json` contains | `axe-core`, `pa11y`, `lighthouse` |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `axe` | `testing.static_analysis_ci` | — | `axe`, `pa11y`, `lighthouse-ci`, `@axe-core` |

### `angular`

| Signal | Values |
|--------|--------|
| Manifests | `angular.json` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | — | `ng lint` |
| `static` | `testing.static_analysis_ci` | — | `ng lint` |
| `test` | `testing.framework` | — | `ng test` |
| `types_ci` | `testing.type_checking_ci` | — | `ng build` |

### `astro`

| Signal | Values |
|--------|--------|
| Manifests | `astro.config.mjs`, `astro.config.ts`, `astro.config.js` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `types` | `local_development.type_checker` | — | `astro check` |
| `types_ci` | `testing.type_checking_ci` | — | `astro check`, `astro build` |

### `css_tooling`

| Signal | Values |
|--------|--------|
| Manifests | `tailwind.config.js`, `tailwind.config.ts`, `postcss.config.js`, `postcss.config.cjs` |
| Path contains | — |
| Basename any | `.stylelintrc`, `stylelint.config.js` |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `format` | `local_development.formatter` | `prettier.config.js` | `prettier` |
| `lint` | `local_development.linter` | `.stylelintrc`, `.stylelintrc.json`, `stylelint.config.js` | `stylelint` |

### `cypress`

| Signal | Values |
|--------|--------|
| Manifests | `cypress.config.ts`, `cypress.config.js`, `cypress.json` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `e2e` | `testing.framework` | `cypress.config.ts`, `cypress.config.js` | `cypress run`, `cypress open` |
| `ui` | `testing.ui` | `cypress.config.ts`, `cypress.config.js`, `cypress.json` | `cypress run`, `cypress open` |

### `nextjs`

| Signal | Values |
|--------|--------|
| Manifests | `next.config.js`, `next.config.mjs`, `next.config.ts` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | — | `next lint` |
| `static` | `testing.static_analysis_ci` | — | `next lint` |
| `types_ci` | `testing.type_checking_ci` | — | `next build`, `tsc` |

### `nuxt`

| Signal | Values |
|--------|--------|
| Manifests | `nuxt.config.ts`, `nuxt.config.js`, `nuxt.config.mjs` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | — | `nuxi`, `nuxt build` |
| `types_ci` | `testing.type_checking_ci` | — | `nuxi typecheck`, `nuxt build` |

### `playwright`

| Signal | Values |
|--------|--------|
| Manifests | `playwright.config.ts`, `playwright.config.js` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `e2e` | `testing.framework` | `playwright.config.ts`, `playwright.config.js` | `playwright test`, `npx playwright` |
| `ui` | `testing.ui` | `playwright.config.ts`, `playwright.config.js` | `playwright test`, `npx playwright` |

### `react`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | — |
| Basename any | — |
| `package.json` contains | `"react"` |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | — | `eslint-plugin-react` |
| `test` | `testing.framework` | — | `react-scripts test`, `@testing-library/react` |

### `remix`

| Signal | Values |
|--------|--------|
| Manifests | `remix.config.js`, `remix.config.ts` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | `@remix-run/` |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `test` | `testing.framework` | — | `vitest`, `playwright` |
| `types_ci` | `testing.type_checking_ci` | — | `remix vite:build`, `tsc` |

### `svelte`

| Signal | Values |
|--------|--------|
| Manifests | `svelte.config.js`, `svelte.config.ts` |
| Path contains | `.svelte` |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `test` | `testing.framework` | — | `vitest` |
| `types` | `local_development.type_checker` | — | `svelte-check` |
| `types_ci` | `testing.type_checking_ci` | — | `svelte-check` |

### `vite`

| Signal | Values |
|--------|--------|
| Manifests | `vite.config.ts`, `vite.config.js`, `vite.config.mjs` |
| Path contains | — |
| Basename any | — |
| `package.json` contains | — |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `test` | `testing.framework` | — | `vitest` |
| `types_ci` | `testing.type_checking_ci` | — | `vite build`, `tsc` |

### `vue`

| Signal | Values |
|--------|--------|
| Manifests | — |
| Path contains | `.vue` |
| Basename any | — |
| `package.json` contains | `"vue"` |

| Tooling id | Maps to rule | Example configs | Example CI needles |
|------------|--------------|-----------------|--------------------|
| `lint` | `local_development.linter` | — | `eslint`, `vue-tsc` |
| `types` | `local_development.type_checker` | — | `vue-tsc` |
| `types_ci` | `testing.type_checking_ci` | — | `vue-tsc` |

## Related

- [Hygiene controls](hygiene-controls.md) — rules packs feed
- [README — Language and web packs](../README.md#language-and-web-packs)
- [examples/apo.toml](../examples/apo.toml)
- [Glossary](glossary.md)
- [Design notes](design.md)
