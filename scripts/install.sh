#!/bin/sh
# Install apo from GitHub Releases (no Rust toolchain required).
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/baoulo/apo/main/scripts/install.sh | sh
#   APO_VERSION=v0.3.1 sh scripts/install.sh
set -eu

REPO="${APO_REPO:-baoulo/apo}"
INSTALL_DIR="${APO_INSTALL_DIR:-${HOME}/.local/bin}"
VERSION="${APO_VERSION:-}"
tmp=""

cleanup() {
  if [ -n "${tmp}" ]; then
    rm -rf "${tmp}"
  fi
}
trap cleanup EXIT

detect_platform() {
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "${os}" in
    linux) os="linux" ;;
    darwin) os="macos" ;;
    mingw*|msys*|cygwin*) os="windows" ;;
    *) echo "error: unsupported OS: $(uname -s)" >&2; exit 1 ;;
  esac
  case "${arch}" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) echo "error: unsupported architecture: ${arch}" >&2; exit 1 ;;
  esac
  platform="${os}-${arch}"
}

latest_tag() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1
}

download() {
  curl -fsSL "$1" -o "$2"
}

verify_checksum() {
  archive="$1"
  sums="$2"
  base="$(basename "${archive}")"
  expected="$(awk -v f="${base}" '$2 == f { print $1; exit }' "${sums}" || true)"
  if [ -z "${expected}" ]; then
    echo "warning: no checksum entry for ${base}; skipping verification" >&2
    return 0
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "${archive}" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "${archive}" | awk '{print $1}')"
  else
    echo "warning: no sha256 tool found; skipping verification" >&2
    return 0
  fi
  if [ "${actual}" != "${expected}" ]; then
    echo "error: checksum mismatch for ${base}" >&2
    exit 1
  fi
}

main() {
  if [ -z "${VERSION}" ]; then
    VERSION="$(latest_tag)"
  fi
  VERSION="${VERSION#v}"
  tag="v${VERSION}"
  detect_platform
  case "${platform}" in
    windows-*) ext="zip" ;;
    *) ext="tar.gz" ;;
  esac
  archive_name="apo-${tag}-${platform}.${ext}"

  tmp="$(mktemp -d)"

  download "https://github.com/${REPO}/releases/download/${tag}/${archive_name}" \
    "${tmp}/${archive_name}"
  download "https://github.com/${REPO}/releases/download/${tag}/SHA256SUMS" \
    "${tmp}/SHA256SUMS" || true
  if [ -f "${tmp}/SHA256SUMS" ]; then
    verify_checksum "${tmp}/${archive_name}" "${tmp}/SHA256SUMS"
  fi

  mkdir -p "${INSTALL_DIR}"
  if [ "${ext}" = "tar.gz" ]; then
    tar -xzf "${tmp}/${archive_name}" -C "${tmp}"
    install -m 0755 "${tmp}/apo" "${INSTALL_DIR}/apo"
  else
    echo "error: use the Windows zip from Releases on Windows" >&2
    exit 1
  fi
  echo "installed ${INSTALL_DIR}/apo (${tag})"
}

main "$@"
