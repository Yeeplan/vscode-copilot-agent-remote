#!/usr/bin/env node
// scripts/preview.mjs
// Runs `vite preview --host` and prints a terminal QR code for the Network URL.

import { spawn } from 'child_process';
import qrcode from 'qrcode-terminal';

const vite = spawn(
  'node',
  ['node_modules/.bin/vite', 'preview', '--host'],
  { stdio: ['inherit', 'pipe', 'pipe'] }
);

let qrShown = false;

function handleLine(line) {
  process.stdout.write(line + '\n');
  if (!qrShown) {
    const match = line.match(/Network:\s+(https?:\/\/\S+)/);
    if (match) {
      const url = match[1];
      qrShown = true;
      console.log('\n  扫码在手机上打开：\n');
      qrcode.generate(url, { small: true });
      console.log(`  ${url}\n`);
    }
  }
}

let stdoutBuf = '';
vite.stdout.on('data', (chunk) => {
  stdoutBuf += chunk.toString();
  const lines = stdoutBuf.split('\n');
  stdoutBuf = lines.pop();
  lines.forEach(handleLine);
});

let stderrBuf = '';
vite.stderr.on('data', (chunk) => {
  stderrBuf += chunk.toString();
  const lines = stderrBuf.split('\n');
  stderrBuf = lines.pop();
  lines.forEach((l) => process.stderr.write(l + '\n'));
});

vite.on('close', (code) => process.exit(code ?? 0));
