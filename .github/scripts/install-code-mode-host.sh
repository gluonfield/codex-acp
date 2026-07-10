#!/usr/bin/env bash
set -euo pipefail

case "${TARGET:?}" in
  aarch64-apple-darwin)
    source=codex-code-mode-host-aarch64-apple-darwin
    sha256=6cf9282430befe541369c7cb2804604a7f0dd9416f3a3241e3676db22022a246
    ;;
  x86_64-apple-darwin)
    source=codex-code-mode-host-x86_64-apple-darwin
    sha256=6fd2b21d9737f90d9cd047da717d378e58009c0c069b5ecd4fb86ebcfef52d1f
    ;;
  aarch64-unknown-linux-gnu)
    source=codex-code-mode-host-aarch64-unknown-linux-musl
    sha256=2ab25695f61ac23a71e467425322a1f197ea52e9da9aa8e0cbc339d661c6d16a
    ;;
  x86_64-unknown-linux-gnu)
    source=codex-code-mode-host-x86_64-unknown-linux-musl
    sha256=26d9c65c5a947c2bf489513ef7f81e027b0c96dc15e2781de6eed5e02a18993d
    ;;
  aarch64-pc-windows-msvc)
    source=codex-code-mode-host-aarch64-pc-windows-msvc.exe
    sha256=3f12d2aa931bc1bb97c29388695c4bff66710d3698ea1780eb124781903d065d
    ;;
  x86_64-pc-windows-msvc)
    source=codex-code-mode-host-x86_64-pc-windows-msvc.exe
    sha256=68f4274436fabac8aaec43a3a8823c8c817f7bdd99b8e48533e7614d57278006
    ;;
  *)
    echo "unsupported code-mode host target: $TARGET" >&2
    exit 1
    ;;
esac

archive=$(mktemp)
trap 'rm -f "$archive"' EXIT
codex_tag=$(sed -n 's/^codex-core = .*tag = "\([^"]*\)".*/\1/p' Cargo.toml)
curl -fsSL "https://github.com/openai/codex/releases/download/${codex_tag:?}/${source}.tar.gz" -o "$archive"
if command -v sha256sum >/dev/null; then
  actual=$(sha256sum "$archive" | awk '{print $1}')
else
  actual=$(shasum -a 256 "$archive" | awk '{print $1}')
fi
if [[ "$actual" != "$sha256" ]]; then
  echo "code-mode host checksum mismatch for $TARGET" >&2
  exit 1
fi

release="target/${TARGET}/release"
tar xzf "$archive" -C "$release" "$source"
target=codex-code-mode-host
if [[ "$source" == *.exe ]]; then
  target+=.exe
fi
mv "$release/$source" "$release/$target"
chmod 0755 "$release/$target"
