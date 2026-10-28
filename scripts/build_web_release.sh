#!/usr/bin/env bash
# Сборка web-версии с минимальным размером (см. README, раздел «Web»).
#
# Использование:  scripts/build_web_release.sh
# Требования:     cargo, trunk, wasm-opt (https://github.com/WebAssembly/binaryen)
#
# Шаги:
#   1. trunk build --release      — cargo (opt-level=z, lto=fat, strip) + wasm-bindgen;
#   2. wasm-opt -Oz               — дополнительное сжатие байткода (~15%);
#   3. brotli (node)              — .br-копии файлов для отдачи с Content-Encoding: br.
#
# Требования: cargo, trunk, wasm-opt (https://github.com/WebAssembly/binaryen),
#             Node.js 18+ (для brotli и локального сервера).
# Результат: dist/web/ (~16 МБ wasm, ~3 МБ в brotli; до всех оптимизаций было 51 МБ).
# Тест отдачи: node scripts/serve_web.mjs

set -euo pipefail
cd "$(dirname "$0")/.."

# Флаги proposals: код от wasm-bindgen использует bulk-memory, sign-ext,
# nontrapping-float-to-int и reference-types — всё это базово поддерживается
# современными браузерами (Chrome 75+, Firefox 79+, Safari 15+).
OPT_FLAGS="--enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int --enable-reference-types"

echo "==> trunk build --release"
trunk build --release

echo "==> wasm-opt -Oz"
wasm=""
for f in dist/web/*_bg.wasm; do
    wasm="$f"
done
[ -n "$wasm" ] || { echo "wasm не найден в dist/web"; exit 1; }

wasm-opt -Oz $OPT_FLAGS -o "$wasm.opt" "$wasm"
mv "$wasm.opt" "$wasm"

echo "==> brotli"
node scripts/brotli_compress.mjs dist/web

echo "==> готово:"
ls -la dist/web/
