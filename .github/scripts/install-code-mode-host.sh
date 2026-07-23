#!/usr/bin/env bash
set -euo pipefail

case "${TARGET:?}" in
  aarch64-apple-darwin)
    source=codex-code-mode-host-aarch64-apple-darwin
    sha256=75f9306834aa8913b5c1f91ff72f1f6b9441e5a92cd5d64b8e605cf54668460c
    ;;
  x86_64-apple-darwin)
    source=codex-code-mode-host-x86_64-apple-darwin
    sha256=2628a7925ff13704126693a2d964fb6d9433a70f5b10c7a966dad3629b55a939
    ;;
  aarch64-unknown-linux-gnu)
    source=codex-code-mode-host-aarch64-unknown-linux-musl
    sha256=22b5862c7206bc944f59402dbab4b4169e381ae8a68f0144a9ba7b61bcf3dd39
    ;;
  x86_64-unknown-linux-gnu)
    source=codex-code-mode-host-x86_64-unknown-linux-musl
    sha256=ac23177956c30cc1f9f180c27bd80f5bb5b76780db55fb94dcc22644d490852e
    ;;
  aarch64-pc-windows-msvc)
    source=codex-code-mode-host-aarch64-pc-windows-msvc.exe
    sha256=17247aacee9e4f76d9e2693324fe6b1a66e053923e5e8a2532da6e797483cd2c
    ;;
  x86_64-pc-windows-msvc)
    source=codex-code-mode-host-x86_64-pc-windows-msvc.exe
    sha256=1fefa1d74e462dfdc081b24af50118ae7ffc8bcbe479ec1cf043ba5fd574cc87
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
