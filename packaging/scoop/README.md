# Scoop manifest

Publish `apo.json` to [`baoulo/scoop-bucket`](https://github.com/baoulo/scoop-bucket).

Checksums are filled from the GitHub Release `SHA256SUMS` file by
[`scripts/update-packaging-checksums.sh`](../../scripts/update-packaging-checksums.sh)
(also run automatically in the Release workflow after the GitHub Release is created):

```bash
scripts/update-packaging-checksums.sh v0.3.2
```

```powershell
scoop bucket add baoulo https://github.com/baoulo/scoop-bucket
scoop install apo
```
