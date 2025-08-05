#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const packageJson = require('./package.json');
const version = packageJson.version;

// Platform detection
const platform = process.platform;
const arch = process.arch;

// Map Node.js platform/arch to Rust target triples
const targets = {
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'win32-x64': 'x86_64-pc-windows-msvc',
};

const platformKey = `${platform}-${arch}`;
const target = targets[platformKey];

if (!target) {
  console.error(`Unsupported platform: ${platform} ${arch}`);
  process.exit(1);
}

const binName = platform === 'win32' ? 'sniff.exe' : 'sniff';
const binDir = path.join(__dirname, 'bin');
const binPath = path.join(binDir, binName);

// Create bin directory if it doesn't exist
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

// For now, we'll build from source since we don't have GitHub releases set up yet
console.log('Building sniff-check from source...');

try {
  // Check if Rust is installed
  execSync('cargo --version', { stdio: 'pipe' });
  
  // Build the binary
  console.log('Building Rust binary...');
  execSync('cargo build --release', { stdio: 'inherit' });
  
  // Copy the binary to bin directory
  const sourceBinary = path.join(__dirname, 'target', 'release', binName);
  if (fs.existsSync(sourceBinary)) {
    fs.copyFileSync(sourceBinary, binPath);
    
    // Make it executable on Unix systems
    if (platform !== 'win32') {
      fs.chmodSync(binPath, 0o755);
    }
    
    console.log('✅ sniff-check installed successfully!');
    console.log(`Binary location: ${binPath}`);
    console.log('Run "sniff --help" to get started.');
  } else {
    throw new Error('Built binary not found');
  }
  
} catch (error) {
  console.error('❌ Failed to build sniff-check');
  console.error('Make sure you have Rust installed: https://rustup.rs/');
  console.error('Error:', error.message);
  process.exit(1);
}

// TODO: In the future, download pre-built binaries from GitHub releases
// const downloadUrl = `https://github.com/rizkyviali/sniff-check/releases/download/v${version}/sniff-${target}${platform === 'win32' ? '.exe' : ''}`;
// console.log(`Downloading sniff-check v${version} for ${platformKey}...`);
// downloadBinary(downloadUrl, binPath);

function downloadBinary(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    
    https.get(url, (response) => {
      if (response.statusCode === 200) {
        response.pipe(file);
        file.on('finish', () => {
          file.close();
          // Make executable on Unix systems
          if (process.platform !== 'win32') {
            fs.chmodSync(dest, 0o755);
          }
          console.log('✅ sniff-check installed successfully!');
          resolve();
        });
      } else if (response.statusCode === 302 || response.statusCode === 301) {
        // Handle redirect
        downloadBinary(response.headers.location, dest).then(resolve).catch(reject);
      } else {
        reject(new Error(`Failed to download: ${response.statusCode}`));
      }
    }).on('error', reject);
  });
}