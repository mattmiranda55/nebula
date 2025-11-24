#!/usr/bin/env node
const { spawn } = require('child_process');
const http = require('http');

function waitForUrl(url, timeout = 15000) {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    const check = () => {
      http.get(url, (res) => {
        resolve(url);
      }).on('error', (err) => {
        if (Date.now() - start > timeout) {
          reject(new Error('Timeout waiting for ' + url));
        } else {
          setTimeout(check, 200);
        }
      });
    };
    check();
  });
}

const vite = spawn('bun', ['run', 'dev'], { stdio: ['ignore', 'pipe', 'inherit'] });

vite.stdout.on('data', (chunk) => {
  const s = chunk.toString();
  process.stdout.write(s);
});

vite.on('error', (err) => {
  console.error('Failed to start Vite dev server:', err);
});

(async () => {
  const url = 'http://localhost:5173';
  try {
    await waitForUrl(url, 20000);
    // Launch electron with renderer URL set
    const env = Object.assign({}, process.env, { ELECTRON_RENDERER_URL: url });
    const electron = spawn('bunx', ['electron', '.'], { stdio: 'inherit', env });

    electron.on('exit', (code) => {
      process.exit(code);
    });
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
