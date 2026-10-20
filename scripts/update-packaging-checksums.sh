#!/usr/bin/env bash
# Fill Homebrew and Scoop packaging checksums from a GitHub Release SHA256SUMS.
#
# Usage:
#   scripts/update-packaging-checksums.sh v0.3.2
#   scripts/update-packaging-checksums.sh 0.3.2 --sums /path/to/SHA256SUMS
#   APO_VERSION=v0.3.2 scripts/update-packaging-checksums.sh
#
# Without --sums, downloads:
#   https://github.com/<repo>/releases/download/vX.Y.Z/SHA256SUMS
set -euo pipefail

REPO="${APO_REPO:-baoulo/apo}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOMEBREW_RB="${ROOT}/packaging/homebrew/apo.rb"
SCOOP_JSON="${ROOT}/packaging/scoop/apo.json"

version=""
sums_path=""

usage() {
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage ;;
    --sums)
      sums_path="${2:?--sums requires a path}"
      shift 2
      ;;
    --repo)
      REPO="${2:?--repo requires owner/name}"
      shift 2
      ;;
    -*)
      echo "error: unknown option: $1" >&2
      usage
      ;;
    *)
      if [[ -n "${version}" ]]; then
        echo "error: unexpected argument: $1" >&2
        usage
      fi
      version="$1"
      shift
      ;;
  esac
done

version="${version:-${APO_VERSION:-}}"
if [[ -z "${version}" ]]; then
  echo "error: version required (e.g. v0.3.2)" >&2
  usage
fi

# Normalize: tag form vX.Y.Z and bare X.Y.Z
version="${version#v}"
tag="v${version}"

hash_for() {
  local artifact="$1"
  local line hash name
  while IFS= read -r line || [[ -n "${line}" ]]; do
    [[ -z "${line}" || "${line}" =~ ^# ]] && continue
    hash="$(awk '{print $1}' <<<"${line}")"
    name="$(awk '{print $2}' <<<"${line}")"
    name="${name#\*}"
    if [[ "${name}" == "${artifact}" ]]; then
      printf '%s\n' "${hash}"
      return 0
    fi
  done <"${sums_file}"
  return 1
}

require_hash() {
  local artifact="$1"
  local hash
  if ! hash="$(hash_for "${artifact}")"; then
    echo "error: missing ${artifact} in ${sums_file}" >&2
    exit 1
  fi
  if [[ ! "${hash}" =~ ^[0-9a-fA-F]{64}$ ]]; then
    echo "error: invalid sha256 for ${artifact}: ${hash}" >&2
    exit 1
  fi
  printf '%s\n' "${hash}"
}

tmp_sums=""
cleanup() {
  if [[ -n "${tmp_sums}" ]]; then
    rm -f "${tmp_sums}"
  fi
}
trap cleanup EXIT

if [[ -n "${sums_path}" ]]; then
  sums_file="${sums_path}"
  if [[ ! -f "${sums_file}" ]]; then
    echo "error: SHA256SUMS not found: ${sums_file}" >&2
    exit 1
  fi
else
  tmp_sums="$(mktemp)"
  sums_file="${tmp_sums}"
  url="https://github.com/${REPO}/releases/download/${tag}/SHA256SUMS"
  echo "Downloading ${url}"
  curl -fsSL "${url}" -o "${sums_file}"
fi

mac_arm="$(require_hash "apo-${tag}-macos-aarch64.tar.gz")"
mac_intel="$(require_hash "apo-${tag}-macos-x86_64.tar.gz")"
linux_arm="$(require_hash "apo-${tag}-linux-aarch64.tar.gz")"
linux_intel="$(require_hash "apo-${tag}-linux-x86_64.tar.gz")"
windows="$(require_hash "apo-${tag}-windows-x86_64.zip")"

if [[ ! -f "${HOMEBREW_RB}" ]]; then
  echo "error: missing ${HOMEBREW_RB}" >&2
  exit 1
fi
if [[ ! -f "${SCOOP_JSON}" ]]; then
  echo "error: missing ${SCOOP_JSON}" >&2
  exit 1
fi

# Preserve formula metadata above the version line from the existing file.
desc="$(sed -n 's/^  desc "\(.*\)"/\1/p' "${HOMEBREW_RB}" | head -1)"
homepage="$(sed -n 's/^  homepage "\(.*\)"/\1/p' "${HOMEBREW_RB}" | head -1)"
license="$(sed -n 's/^  license "\(.*\)"/\1/p' "${HOMEBREW_RB}" | head -1)"
desc="${desc:-APO — Engineering Evidence Platform. Repository hygiene, knowledge, and AI evidence.}"
homepage="${homepage:-https://github.com/${REPO}}"
license="${license:-MIT}"

cat >"${HOMEBREW_RB}" <<EOF
# Homebrew Formula for apo
#
# Source of truth in-repo. Checksums are filled by
# scripts/update-packaging-checksums.sh from the GitHub Release SHA256SUMS.
# Copy to baoulo/homebrew-tap as Formula/apo.rb after each release.
class Apo < Formula
  desc "${desc}"
  homepage "${homepage}"
  version "${version}"
  license "${license}"

  on_macos do
    on_arm do
      url "https://github.com/${REPO}/releases/download/v#{version}/apo-v#{version}-macos-aarch64.tar.gz"
      sha256 "${mac_arm}"
    end
    on_intel do
      url "https://github.com/${REPO}/releases/download/v#{version}/apo-v#{version}-macos-x86_64.tar.gz"
      sha256 "${mac_intel}"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/${REPO}/releases/download/v#{version}/apo-v#{version}-linux-aarch64.tar.gz"
      sha256 "${linux_arm}"
    end
    on_intel do
      url "https://github.com/${REPO}/releases/download/v#{version}/apo-v#{version}-linux-x86_64.tar.gz"
      sha256 "${linux_intel}"
    end
  end

  def install
    bin.install "apo"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/apo --version")
  end
end
EOF

scoop_desc="$(sed -n 's/^  "description": "\(.*\)",/\1/p' "${SCOOP_JSON}" | head -1)"
scoop_homepage="$(sed -n 's/^  "homepage": "\(.*\)",/\1/p' "${SCOOP_JSON}" | head -1)"
scoop_license="$(sed -n 's/^  "license": "\(.*\)",/\1/p' "${SCOOP_JSON}" | head -1)"
scoop_desc="${scoop_desc:-APO — Engineering Evidence Platform. Repository hygiene, knowledge, and AI evidence.}"
scoop_homepage="${scoop_homepage:-https://github.com/${REPO}}"
scoop_license="${scoop_license:-MIT}"

cat >"${SCOOP_JSON}" <<EOF
{
  "version": "${version}",
  "description": "${scoop_desc}",
  "homepage": "${scoop_homepage}",
  "license": "${scoop_license}",
  "architecture": {
    "64bit": {
      "url": "https://github.com/${REPO}/releases/download/v${version}/apo-v${version}-windows-x86_64.zip",
      "hash": "${windows}"
    }
  },
  "bin": "apo.exe",
  "checkver": {
    "github": "https://github.com/${REPO}"
  },
  "autoupdate": {
    "architecture": {
      "64bit": {
        "url": "https://github.com/${REPO}/releases/download/v\$version/apo-v\$version-windows-x86_64.zip",
        "hash": {
          "url": "https://github.com/${REPO}/releases/download/v\$version/SHA256SUMS"
        }
      }
    }
  }
}
EOF

echo "Updated ${HOMEBREW_RB}"
echo "Updated ${SCOOP_JSON}"
echo "version ${version} checksums applied from ${sums_file}"
