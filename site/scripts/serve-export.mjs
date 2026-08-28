// SPDX-License-Identifier: Apache-2.0

import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, resolve, sep } from 'node:path';

const root = resolve(process.argv[2] ?? 'site/out');
const port = Number.parseInt(process.argv[3] ?? '4174', 10);
const host = '127.0.0.1';

if (!Number.isInteger(port) || port < 1 || port > 65535) {
  console.error('serve-export: port must be an integer from 1 through 65535');
  process.exit(2);
}

const contentTypes = new Map([
  ['.css', 'text/css; charset=utf-8'],
  ['.html', 'text/html; charset=utf-8'],
  ['.ico', 'image/x-icon'],
  ['.jpeg', 'image/jpeg'],
  ['.jpg', 'image/jpeg'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.png', 'image/png'],
  ['.svg', 'image/svg+xml'],
  ['.txt', 'text/plain; charset=utf-8'],
  ['.webmanifest', 'application/manifest+json; charset=utf-8'],
  ['.woff', 'font/woff'],
  ['.woff2', 'font/woff2'],
]);

function withinRoot(path) {
  return path === root || path.startsWith(`${root}${sep}`);
}

async function firstFile(paths) {
  for (const path of paths) {
    if (!withinRoot(path)) continue;
    try {
      if ((await stat(path)).isFile()) return path;
    } catch (error) {
      if (error.code !== 'ENOENT' && error.code !== 'ENOTDIR') throw error;
    }
  }
  return null;
}

const server = createServer(async (request, response) => {
  try {
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      response.writeHead(405, { Allow: 'GET, HEAD' });
      response.end();
      return;
    }

    let pathname;
    try {
      pathname = decodeURIComponent(new URL(request.url, `http://${host}`).pathname);
    } catch {
      response.writeHead(400);
      response.end();
      return;
    }

    const relative = pathname.replace(/^\/+/, '');
    const candidates = pathname === '/'
      ? [resolve(root, 'index.html')]
      : [
          resolve(root, relative),
          resolve(root, `${relative}.html`),
          resolve(root, relative, 'index.html'),
        ];
    let path = await firstFile(candidates);
    const status = path === null ? 404 : 200;
    path ??= resolve(root, '404.html');

    const body = await readFile(path);
    response.writeHead(status, {
      'Cache-Control': 'no-store',
      'Content-Length': body.length,
      'Content-Type': contentTypes.get(extname(path).toLowerCase()) ?? 'application/octet-stream',
    });
    response.end(request.method === 'HEAD' ? undefined : body);
  } catch (error) {
    console.error(`serve-export: ${error.message}`);
    response.writeHead(500);
    response.end();
  }
});

server.listen(port, host, () => {
  console.log(`serve-export: ${root} at http://${host}:${port}`);
});
