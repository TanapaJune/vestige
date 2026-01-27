#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

const platform = os.platform();
const binaryName = platform === 'win32' ? 'vestige.exe' : 'vestige';
const binaryPath = path.join(__dirname, binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error('Error: vestige CLI binary not found.');
  console.error(`Expected at: ${binaryPath}`);
  console.error('');
  console.error('Try reinstalling: npm install -g @vestige/mcp');
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
});

child.on('error', (err) => {
  console.error('Failed to start vestige:', err.message);
  process.exit(1);
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
