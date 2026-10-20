# Homebrew formula

Source of truth for the formula. Publish by copying to
[`baoulo/homebrew-tap`](https://github.com/baoulo/homebrew-tap) as `Formula/apo.rb`.

Checksums are filled from the GitHub Release `SHA256SUMS` file by
[`scripts/update-packaging-checksums.sh`](../../scripts/update-packaging-checksums.sh)
(also run automatically in the Release workflow after the GitHub Release is created).
The same workflow smoke-tests this formula on macOS with
`brew install --formula ./packaging/homebrew/apo.rb` before crates.io publish.

```bash
scripts/update-packaging-checksums.sh v0.3.2
```

```bash
brew install baoulo/tap/apo
```
