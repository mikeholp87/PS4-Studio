// Serves the PRODUCTION build (dist/) with the exact Content-Security-Policy from tauri.conf.json,
// so tests catch anything the strict CSP blocks (e.g. eval) that the dev server would hide.
import http from 'node:http'
import { readFileSync, existsSync } from 'node:fs'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('../dist', import.meta.url))
const conf = JSON.parse(readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'))
const csp = conf.app.security.csp
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png' }

http
  .createServer((req, res) => {
    const url = new URL(req.url ?? '/', 'http://x').pathname
    let file = normalize(join(root, url === '/' ? 'index.html' : url))
    if (!file.startsWith(root) || !existsSync(file)) file = join(root, 'index.html')
    res.writeHead(200, { 'Content-Type': types[extname(file)] ?? 'application/octet-stream', 'Content-Security-Policy': csp })
    res.end(readFileSync(file))
  })
  .listen(4173, '127.0.0.1')
