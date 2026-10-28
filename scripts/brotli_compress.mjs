#!/usr/bin/env node
// Brotli-сжатие артефактов web-сборки (dist/web) для отдачи с заголовком
// Content-Encoding: br. Вызывается из scripts/build_web_release.sh.
//
// Использование:  node scripts/brotli_compress.mjs [dir]   (по умолчанию dist/web)
// Результат:      рядом с каждым файлом появляется его .br-копия.

import { readdirSync, readFileSync, writeFileSync, statSync, unlinkSync } from 'node:fs';
import { brotliCompressSync, constants } from 'node:zlib';
import { join, extname } from 'node:path';

const dir = process.argv[2] ?? 'dist/web';
const COMPRESSIBLE = /\.(wasm|js|html|css|json|svg)$/;

// Качество 11 — максимум сжатия; сборка нечастая, размер важнее времени.
const PARAMS = {
    [constants.BROTLI_PARAM_QUALITY]: 11,
    [constants.BROTLI_PARAM_SIZE_HINT]: 0,
};

let totalBefore = 0;
let totalAfter = 0;

for (const name of readdirSync(dir)) {
    if (name.endsWith('.br') || !COMPRESSIBLE.test(extname(name).toLowerCase())) {
        continue;
    }
    const path = join(dir, name);
    const src = readFileSync(path);

    // SIZE_HINT важен для больших wasm: без него brotli резервирует память
    // под маленький вход и жмёт хуже.
    const out = brotliCompressSync(src, {
        params: { ...PARAMS, [constants.BROTLI_PARAM_SIZE_HINT]: src.length },
    });

    const brPath = path + '.br';
    if (statSync(path).size > 0) writeFileSync(brPath, out);

    totalBefore += src.length;
    totalAfter += out.length;
    console.log(
        `${name}: ${(src.length / 1e6).toFixed(2)} МБ -> ${(out.length / 1e6).toFixed(2)} МБ ` +
        `(x${(src.length / out.length).toFixed(2)})`,
    );
}

console.log(`== итого: ${(totalBefore / 1e6).toFixed(2)} МБ -> ${(totalAfter / 1e6).toFixed(2)} МБ`);
