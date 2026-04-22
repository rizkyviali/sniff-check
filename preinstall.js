#!/usr/bin/env node

const path = require("node:path");
const fs = require("node:fs");

const g = process.env.npm_config_global;
const sd = process.env.npm_config_save_dev;
const so = process.env.npm_config_save_optional;
const prod = process.env.NODE_ENV === "production";
const ci =
  !!process.env.CI ||
  !!process.env.VERCEL ||
  !!process.env.NETLIFY ||
  !!process.env.GITHUB_ACTIONS;

// Skip check in production/CI/deployment environments
if (prod || ci) process.exit(0);

// Global install is always fine
if (g) process.exit(0);

// --save-dev or --save-optional flag was passed directly
if (sd || so) process.exit(0);

// Fallback: check parent package.json to detect devDependency placement
// (npm_config_save_dev is only set when --save-dev flag is passed, not during `npm install`)
try {
  const parentPkgPath = path.resolve(__dirname, "../../package.json");
  if (fs.existsSync(parentPkgPath)) {
    const parentPkg = JSON.parse(fs.readFileSync(parentPkgPath, "utf8"));
    const inDev =
      parentPkg.devDependencies && parentPkg.devDependencies["sniff-check"];
    const inOpt =
      parentPkg.optionalDependencies &&
      parentPkg.optionalDependencies["sniff-check"];
    if (inDev || inOpt) process.exit(0);
  }
} catch (_) {
  process.exit(0);
}

console.error(
  "❌ sniff-check should be installed globally (npm install -g sniff-check) or as a devDependency (npm install --save-dev sniff-check)"
);
process.exit(1);
