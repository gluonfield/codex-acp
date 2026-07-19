#!/usr/bin/env bash
set -euo pipefail

case "${TARGET:?}" in
  aarch64-apple-darwin)
    source=codex-code-mode-host-aarch64-apple-darwin
    sha256=7bfd7b3344625be4aad7f3d2a2ac4202f986412842cae11d194d28ca1ebac586
    ;;
  x86_64-apple-darwin)
    source=codex-code-mode-host-x86_64-apple-darwin
    sha256=a7472069c3ee3b9f3afe064af02a72289d8f4baf052d1fe32a799cb43d3d7735
    ;;
  aarch64-unknown-linux-gnu)
    source=codex-code-mode-host-aarch64-unknown-linux-musl
    sha256=7fb9dd606784e0cf239d7d461c72cf86edc20bbdc733d7029298e6c48d230ede
    ;;
  x86_64-unknown-linux-gnu)
    source=codex-code-mode-host-x86_64-unknown-linux-musl
    sha256=37104c43f62719709309d06e69f003e8e8bed1397f4d36476b3b65e25fc04493
    ;;
  aarch64-pc-windows-msvc)
    source=codex-code-mode-host-aarch64-pc-windows-msvc.exe
    sha256=f8d831a58f20942e70576b005aa0ac8f1aa048e03797bab8cef26320605c7cbe
    ;;
  x86_64-pc-windows-msvc)
    source=codex-code-mode-host-x86_64-pc-windows-msvc.exe
    sha256=f7464a0389026bd52161d31e77aac9dca0f7d05c6e14034c95a6fdb43f27d19a
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
