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

// Map Node.js platform/arch to artifact names
const artifactMap = {
  'linux-x64': 'sniff-linux-x64',
  'linux-arm64': 'sniff-linux-arm64', 
  'darwin-x64': 'sniff-macos-x64',
  'darwin-arm64': 'sniff-macos-arm64',
  'win32-x64': 'sniff-windows-x64.exe',
};

const platformKey = `${platform}-${arch}`;
const artifactName = artifactMap[platformKey];

if (!artifactName) {
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

// Download pre-built binary from GitHub releases
const downloadUrl = `https://github.com/rizkyviali/sniff-check/releases/download/v${version}/${artifactName}`;

console.log(`Downloading sniff-check v${version} for ${platformKey}...`);

downloadBinary(downloadUrl, binPath)
  .then(() => {
    console.log('✅ sniff-check installed successfully!');
    console.log(`Binary location: ${binPath}`);
    console.log('Run "sniff --help" to get started.');
  })
  .catch((error) => {
    console.error('❌ Failed to download sniff-check');
    console.error('Falling back to building from source...');
    
    // Fallback to building from source
    try {
      execSync('cargo --version', { stdio: 'pipe' });
      console.log('Building Rust binary...');
      execSync('cargo build --release', { stdio: 'inherit' });
      
      const sourceBinary = path.join(__dirname, 'target', 'release', binName);
      if (fs.existsSync(sourceBinary)) {
        fs.copyFileSync(sourceBinary, binPath);
        
        if (platform !== 'win32') {
          fs.chmodSync(binPath, 0o755);
        }
        
        console.log('✅ sniff-check built and installed successfully!');
        console.log(`Binary location: ${binPath}`);
        console.log('Run "sniff --help" to get started.');
      } else {
        throw new Error('Built binary not found');
      }
    } catch (buildError) {
      console.error('❌ Failed to build from source as well');
      console.error('Make sure you have Rust installed: https://rustup.rs/');
      console.error('Download error:', error.message);
      console.error('Build error:', buildError.message);
      process.exit(1);
    }
  });

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