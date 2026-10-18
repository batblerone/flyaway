#!/usr/bin/env bash
# Builds the browser version of the game into web/dist.
#
# You'll need, once:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <the version this prints if it's missing>
# and optionally binaryen's wasm-opt for a smaller download.
#
# Then serve web/dist with any static file server, for example
#   python3 -m http.server --directory web/dist 8080
# and open http://localhost:8080. WebGPU only works on https or localhost.

set -euo pipefail
cd "$(dirname "$0")/.."

CARGO=${CARGO:-cargo}
TARGET_DIR=${CARGO_TARGET_DIR:-target}
OUT=web/dist

# wasm-bindgen's CLI has to match the library version in Cargo.lock exactly,
# or the generated JS won't line up with the wasm.
want=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"/\1/p')
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}' || true)
if [ "$have" != "$want" ]; then
  echo "Need wasm-bindgen-cli $want (found: ${have:-none}). Install it with:"
  echo "  cargo install wasm-bindgen-cli --version $want"
  exit 1
fi

"$CARGO" build --profile web-release --target wasm32-unknown-unknown

rm -rf "$OUT"
mkdir -p "$OUT"
wasm-bindgen \
  --target web \
  --no-typescript \
  --out-dir "$OUT" \
  --out-name flyaway \
  "$TARGET_DIR/wasm32-unknown-unknown/web-release/flyaway.wasm"

# wasm-opt takes roughly a third off the download. Old versions mangle the
# reference-type tables that wasm-bindgen emits, and the page then dies on
# load with "WebAssembly.Table.grow(): failed to grow table". Ubuntu 24.04's
# binaryen 108 does this; 132 is fine. I haven't pinned down exactly which
# release fixed it, so 120 below is a guess. If a build with wasm-opt won't
# start, try without it first.
opt_version=$(wasm-opt --version 2>/dev/null | sed -n 's/.*version \([0-9]*\).*/\1/p' || true)
if [ -z "$opt_version" ]; then
  echo "(wasm-opt not found, skipping the size pass)"
elif [ "$opt_version" -lt 120 ]; then
  echo "(wasm-opt $opt_version is too old and breaks the output, skipping the size pass)"
else
  # Bevy's wasm uses a few post-MVP features, so tell wasm-opt they're fine.
  wasm-opt -Os --enable-bulk-memory --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-mutable-globals --enable-reference-types \
    "$OUT/flyaway_bg.wasm" -o "$OUT/flyaway_bg.wasm"
fi

cp web/index.html "$OUT/"
cp -r assets "$OUT/assets"

echo
echo "Built $OUT ($(du -sh "$OUT" | cut -f1))."
echo "Serve it with: python3 -m http.server --directory $OUT 8080"
