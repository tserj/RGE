#!/usr/bin/env node
// Локальный сервер для теста web-сборки с brotli.
//
// Отдаёт файлы из dist/web: если клиент прислал Accept-Encoding: br и рядом
// с файлом лежит его .br-копия (см. scripts/brotli_compress.mjs), отдаётся
// сжатая версия с заголовком Content-Encoding: br — браузер распаковывает
// её прозрачно, клиенту ничего делать не нужно.
//
// Использование:  node scripts/serve_web.mjs [порт]   (по умолчанию 8088)

import { createServer } from 'node:http';
import { createReadStream, existsSync, statSync } from 'node:fs';
import { extname, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(fileURLToPath(new URL('../dist/web/', import.meta.url)));
const PORT = Number(process.argv[2] ?? process.env.PORT ?? 8088);

const MIME = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'text/javascript; charset=utf-8',
    '.wasm': 'application/wasm',
    '.json': 'application/json',
    '.svg': 'image/svg+xml',
    '.css': 'text/css; charset=utf-8',
};

createServer((req, res) => {
    // Нормализуем путь, срезаем ведущие разделители (на Windows '\x'
    // означает корень диска!) и запрещаем выход за пределы ROOT.
    let urlPath = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    if (urlPath.endsWith('/')) urlPath += 'index.html';
    const rel = normalize(urlPath).replace(/^([\\/])+/, '');
    if (rel.startsWith('.')) {
        res.writeHead(403).end('forbidden');
        return;
    }
    const file = resolve(ROOT, rel);
    if (file !== ROOT && !file.startsWith(ROOT + sep)) {
        res.writeHead(403).end('forbidden');
        return;
    }

    const acceptsBr = /\bbr\b/.test(req.headers['accept-encoding'] ?? '');
    const brFile = file + '.br';

    if (acceptsBr && existsSync(brFile)) {
        // Сжатая версия: браузер распакует сам (Content-Encoding: br).
        res.writeHead(200, {
            'Content-Type': MIME[extname(file)] ?? 'application/octet-stream',
            'Content-Encoding': 'br',
            'Vary': 'Accept-Encoding',
            'Content-Length': statSync(brFile).size,
        });
        createReadStream(brFile).pipe(res);
        console.log(`br   ${urlPath}`);
    } else if (existsSync(file)) {
        res.writeHead(200, {
            'Content-Type': MIME[extname(file)] ?? 'application/octet-stream',
            'Content-Length': statSync(file).size,
        });
        createReadStream(file).pipe(res);
        console.log(`raw  ${urlPath}`);
    } else {
        res.writeHead(404).end('not found');
        console.log(`404  ${urlPath}`);
    }
}).listen(PORT, () => {
    console.log(`serve ${ROOT}`);
    console.log(`http://localhost:${PORT}/  (brotli: включён, при наличии .br)`);
});
