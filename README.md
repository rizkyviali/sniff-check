# sniff-check

[![npm version](https://badge.fury.io/js/sniff-check.svg)](https://badge.fury.io/js/sniff-check)
[![Downloads](https://img.shields.io/npm/dm/sniff-check.svg)](https://www.npmjs.com/package/sniff-check)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Opinionated TypeScript/Next.js Development Toolkit

A comprehensive, high-performance Rust CLI tool that enforces opinionated code quality standards for TypeScript/Next.js projects. Built for speed, reliability, and developer productivity with advanced performance optimizations and comprehensive testing.

[![Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/rizkyviali)

## 🔥 Core Philosophy

- **Files over 100 lines are considered "smelly code"** and should be refactored
- **TypeScript 'any' usage is forbidden** - strict typing enforced
- **Clean separation** between utilities and components
- **Zero-tolerance** for unused imports and dead code
- **Comprehensive pre-deployment validation**
- **Performance-first architecture** - optimized for large codebases

## ⚡ Performance Features

- **Smart File Discovery** - Automatically excludes binary files, images, and build artifacts
- **Parallel Processing** - Multi-threaded analysis for projects with 20+ files
- **Memory-Mapped I/O** - Efficient handling of large files (1MB+) using memory mapping
- **Intelligent Caching** - Reduces redundant file system operations
- **Performance Monitoring** - Built-in timing and metrics (use `SNIFF_PERF_DEBUG=1`)
- **Optimized Regex Engine** - Shared pattern compilation for faster analysis

## 🚀 Quick Start

```bash
# Install globally via npm (Recommended)
npm install -g sniff-check

# Or use without installation
npx sniff-check

# Or add to your project
npm install --save-dev sniff-check
```

### Alternative Installation Methods

<details>
<summary>Via Cargo (Rust)</summary>

```bash
# Install from crates.io
cargo install sniff-check

# Or build from source
git clone https://github.com/rizkyviali/sniff-check
cd sniff-check
cargo build --release
```
</details>

### Requirements
- **Node.js 18+** (for npm installation)
- **Rust 1.70+** (only for cargo installation)

## 📖 Usage

### 🎯 Most Common Commands

```bash
# 🔍 Find large files that need refactoring (>100 lines)
sniff large

# 🧩 Analyze and split large components
sniff components

# 🚫 Detect unused AND broken imports
sniff imports  

# 📝 Check TypeScript code quality ('any' usage, type coverage)
sniff types

# 🚀 Complete pre-deployment validation
sniff deploy
```

### Interactive Menu
```bash
sniff
# or
sniff menu
```

Shows a beautiful terminal menu with all available commands.

### Core Commands

#### 🔍 Large Files Detection
```bash
sniff large
sniff --json large  # JSON output
sniff --quiet large # Quiet mode
```

Scans all TypeScript/JavaScript files and flags files over the threshold as "smelly code". Provides specific refactoring suggestions based on file type (component, service, API, etc.).

**Severity Levels:**
- **Warning** (100-200 lines): Needs attention
- **Error** (200-400 lines): Should be refactored
- **Critical** (400+ lines): Must be refactored immediately

#### 🧩 Component Analysis & Splitting
```bash
sniff components
sniff components --threshold 150  # Custom line threshold
```

Smart analysis of React, Vue, Angular, and Svelte components:
- **Complexity scoring** based on hooks, props, state, and nesting
- **Framework-specific detection** and recommendations
- **Extractable parts identification** (custom hooks, utility functions, sub-components)
- **Refactoring guidance** with specific splitting strategies
- **Multi-concern detection** for components handling too many responsibilities

**Analysis includes:**
- Hook usage patterns (useState, useEffect, custom hooks)
- Props complexity (too many props suggests multiple concerns)
- Component nesting depth and complexity
- Business logic that could be extracted
- UI elements that could become reusable components

#### 📝 TypeScript Quality Check
```bash
sniff types
```

Comprehensive TypeScript analysis:
- Detects 'any' type usage (CRITICAL)
- Finds missing return type annotations
- Identifies @ts-ignore/@ts-expect-error comments
- Calculates type coverage score (0-100%)

#### 🚫 Unused & Broken Imports Detection
```bash
sniff imports
```

Comprehensive analysis of import statements:
- **Unused Imports**: Detects unused default, named, and namespace imports
- **Broken Imports**: Identifies imports referencing non-existent files or uninstalled packages
- **Smart Suggestions**: Provides fix suggestions for broken imports (perfect for refactoring)
- **Refactoring Support**: Instantly spots issues after moving/renaming files
- Handles complex usage patterns (JSX components, type annotations)
- Shows potential bundle size savings
- Supports ES6, CommonJS, and dynamic imports

#### 📦 Bundle Analysis
```bash
sniff bundle
```

Analyzes build output for optimization opportunities:
- Identifies largest chunks and files
- Calculates compression ratios
- Warns about oversized bundles (>2MB total, >500KB per chunk)
- Provides specific optimization recommendations

#### 🚀 Performance Auditing
```bash
sniff perf
```

Comprehensive performance analysis with Lighthouse integration:
- Runs Lighthouse audits on local development servers
- Checks Core Web Vitals (LCP, FID, CLS)
- Analyzes performance, accessibility, best practices, and SEO
- Provides bundle size analysis and optimization recommendations
- Falls back to basic performance checks if Lighthouse unavailable

#### 🧠 Memory Leak Detection
```bash
sniff memory
```

Advanced memory leak pattern detection:
- Scans for common memory leak patterns in TypeScript/JavaScript
- Detects unremoved event listeners, timer leaks, circular references
- Monitors running Node.js processes for high memory usage
- Provides specific cleanup recommendations and best practices

#### 🔧 Environment Validation
```bash
sniff env
```

Complete environment variable validation:
- Checks required environment variables for TypeScript/Next.js projects
- Validates format for URLs, database connections, and Node environments
- Scans .env files for security issues and sensitive data exposure
- Provides environment health score and configuration recommendations

#### 🏗️ Project Context Analysis
```bash
sniff context
```

Comprehensive project structure and context analysis:
- Analyzes project information (name, version, framework, languages)
- Maps directory structure and identifies file purposes
- Detects architectural patterns and organization quality
- Provides insights into project complexity and recommendations
- Supports multiple frameworks: Next.js, React, Vue, Angular, Svelte

#### 🚀 Pre-deployment Pipeline
```bash
sniff deploy
```

Comprehensive pre-deployment validation pipeline:
- Runs all quality checks in sequence (env, types, large files, imports, bundle)
- Provides deployment readiness assessment
- Shows detailed results for each check with timing information
- Fails fast on critical issues, allows warnings for non-blocking problems

#### ⚙️ Configuration Management
```bash
sniff config init      # Initialize default configuration file
sniff config show      # Show current configuration
sniff config validate  # Validate configuration file
sniff config get types # Show configuration for specific command
```

### Output Formats

```bash
# JSON output for programmatic usage
sniff --json large

# Quiet mode for CI environments
sniff --quiet large

# Custom configuration
sniff --config custom.toml large

# Version information
sniff --version

# Help information
sniff --help
sniff large --help

# Performance debugging (shows detailed timing)
SNIFF_PERF_DEBUG=1 sniff large
```

### Performance Monitoring

Set `SNIFF_PERF_DEBUG=1` to see detailed performance breakdowns:

```bash
SNIFF_PERF_DEBUG=1 sniff large

# Output includes:
# --- Performance Report ---
# Total time: 285ms
#   File discovery: 283ms (Δ 283ms)
#   File analysis: 1.3ms (Δ 1.3ms)
#   Summary creation: 600ns (Δ 600ns)
# Files processed: 1247
# Large files found: 23
```

## 🎯 Features

### ✅ Fully Implemented
- **Interactive Menu** - Beautiful terminal UI with categorized commands
- **Large Files Detection** - Find and refactor "smelly code" files with optimized parallel analysis
- **TypeScript Quality Check** - Comprehensive type analysis and scoring
- **Unused Imports Detection** - Clean up dead imports automatically
- **Bundle Analysis** - Optimize build output and bundle sizes
- **Performance Auditing** - Lighthouse integration for performance testing
- **Memory Leak Detection** - Monitor Node.js memory usage patterns
- **Environment Validation** - Check required environment variables
- **Project Context Analysis** - Comprehensive project structure and insights
- **Pre-deployment Pipeline** - Complete validation before deployment
- **Configuration System** - Project-specific settings and overrides

### ⚡ Performance & Architecture
- **Optimized File Walker** - Smart directory filtering and parallel processing
- **Memory-Mapped I/O** - Efficient large file handling (1MB+ files)
- **Shared Utilities** - Eliminated 300+ lines of duplicate code
- **Unified JSON Output** - Standardized responses with metadata and timing
- **Performance Monitoring** - Built-in profiling and debugging tools
- **Comprehensive Test Suite** - Shared test utilities and integration tests
- **Error Handling** - Standardized exit codes and error reporting

## 🛠️ Configuration

Create a `sniff.toml` file in your project root for project-specific settings:

```toml
[large_files]
threshold = 100
excluded_dirs = [
    "node_modules",
    ".next", 
    "dist",
    ".git",
    "target",
    "build",
    "coverage",
    ".cache",
    "tmp",
    "temp",
]
excluded_files = [
    "*.min.js",
    "*.bundle.js", 
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lockb",
    "*.d.ts",
    "*.config.js",
    "*.config.ts",
]

[large_files.severity_levels]
warning = 100
error = 200
critical = 400

[typescript]
strict_any_check = true
allow_ts_ignore = false
require_return_types = true
min_type_coverage = 80.0

[imports]
auto_fix = false
excluded_patterns = [
    "react",
    "@types/*",
]
check_dev_dependencies = true

[bundle]
max_bundle_size_mb = 2.0
max_chunk_size_mb = 0.5
build_dirs = [
    ".next",
    "dist", 
    "build",
    "out",
]
warn_on_large_chunks = true

[performance]
lighthouse_enabled = true
min_performance_score = 75.0
min_accessibility_score = 90.0
server_urls = [
    "http://localhost:3000",
    "http://localhost:3001",
    "http://localhost:8000",
    "http://localhost:8080",
]

[memory]
check_patterns = true
check_processes = true
max_process_memory_mb = 1000.0
pattern_severity_threshold = "high"

[environment]
required_vars = [
    "NODE_ENV",
    "NEXT_PUBLIC_APP_URL",
    "NEXTAUTH_SECRET",
    "NEXTAUTH_URL",
    "DATABASE_URL",
    "VERCEL_URL",
]
check_security = true
allow_empty_values = false
env_files = [
    ".env",
    ".env.local",
    ".env.development", 
    ".env.production",
    ".env.staging",
    ".env.test",
]

# File type classification for enhanced analysis
[file_types]
# Next.js specific patterns
api_routes = ["**/api/**/*.ts", "**/api/**/*.js"]
server_components = ["**/app/**/*.tsx", "**/app/**/page.tsx", "**/app/**/layout.tsx"]
client_components = ["**/*client*.tsx", "**/*client*.ts", "**/*.client.tsx", "**/*.client.ts"]
middleware = ["middleware.ts", "middleware.js", "**/middleware/**/*.ts"]
custom_hooks = ["**/*use*.ts", "**/*use*.tsx", "**/hooks/**/*.ts"]
type_definitions = ["**/*.d.ts", "**/types/**/*.ts", "**/@types/**/*.ts"]
configuration = ["**/*.config.*", "**/config/**/*", "**/*.env*"]
```

Use `sniff config init` to generate a default configuration file, or `sniff config show` to see your current settings.

## 🆕 Recent Updates

**v0.2.2** — Production environment support:

- **🌐 Deployment Compatibility** — Install script detects Vercel/Netlify/CI environments and skips binary download automatically
- **🔧 DevDependency Enforcement** — Preinstall check prevents accidental production installs
- **📚 Deployment Guide** — Added DEPLOY.md with step-by-step publishing instructions

**v0.2.1** — Pre-built binary distribution:

- **🚀 Faster Installation** — Downloads pre-compiled binaries instead of building from source
- **🔄 Graceful Fallback** — Falls back to `cargo build` if binary download fails
- **🌐 Multi-platform** — Automated binaries for Linux, macOS, and Windows (x64 + ARM64)

**v0.2.0** — Component analysis:

- **🧩 New `sniff components` command** — Complexity scoring, extractable parts detection, and refactoring guidance for React/Vue/Angular/Svelte components
- **⚙️ Configurable thresholds** — Bundle and memory commands now adapt to your actual system and framework
- **🔍 Smart server detection** — `sniff perf` auto-detects running dev servers

See [CHANGELOG.md](./CHANGELOG.md) for complete release notes and version history.

## 📊 Example Output

### Large Files Report
```
🔍 Scanning for large files...

📊 Large Files Report
====================

🚨 CRITICAL FILES
  src/components/UserDashboard.tsx (423 lines) - Component
    • Break into smaller sub-components
    • Extract custom hooks for logic
    • Move utility functions to separate files

⚠️  ERROR FILES
  src/services/api.ts (287 lines) - Service
    • Split into multiple service classes
    • Extract interfaces and types
    • Use dependency injection

📈 SUMMARY
─────────
  Files scanned: 156
  Large files found: 8
  Critical: 1
  Errors: 3
  Warnings: 4

💡 TIP: Files over 100 lines are considered 'smelly code' and should be refactored
```

### Imports Analysis Report
```
🔍 Scanning for unused and broken imports...

📊 Imports Analysis Report
==========================

src/components/UserProfile.tsx
  Line 12: import React from 'react';
    🚫 Unused: React

  Line 15: import { validateEmail } from '../utils/validation';
    🚫 Unused: validateEmail

  Line 18: import OldComponent from './old-component';
    💥 File not found: ./old-component
    💡 Suggestion: ./components/UserComponent

  Line 22: import lodash from 'lodash';
    💥 Module not installed: lodash
    💡 Run: npm install lodash

📈 SUMMARY
─────────
  Files scanned: 156
  Total imports: 284
  Unused imports: 12
  Broken imports: 8
  Potential savings: ~12 lines of code

💡 TIP: Remove unused imports to reduce bundle size and improve build performance
🔧 Consider using an IDE extension or linter to automatically remove unused imports
🔧 Fix broken imports to resolve compilation errors
💡 Check if files were moved/renamed, or if packages need to be installed
```

### TypeScript Quality Report
```
🔍 Checking TypeScript type coverage...

📊 TypeScript Quality Report
===========================

🚫 'ANY' TYPE USAGE (CRITICAL)
─────────────────────────────
  src/utils/helpers.ts:42 - Usage of 'any' type detected
    💡 Consider using a more specific type

📈 SUMMARY
─────────
  Files scanned: 89
  Total issues: 12
  'any' usage: 5
  Missing return types: 4
  TS suppressions: 3

  Type Coverage Score: 87.2%

🚫 CRITICAL: Usage of 'any' type is strictly forbidden!
   All 'any' types must be replaced with specific types.
```

### Project Context Report
```
🔍 Analyzing project structure and context...

📊 Project Context Report
========================

🏗️  PROJECT OVERVIEW
─────────────────────
  Name: my-nextjs-app
  Version: 1.2.0
  Description: Modern e-commerce platform
  Framework: NextJs
  Languages: [TypeScript, JavaScript, CSS, JSON]
  Total Files: 247
  Total Lines: 12,485

📁 PROJECT STRUCTURE
──────────────────────
  Key Directories:
    • src/components (45 files, 3,241 lines)
      Purpose: Components | File types: tsx, ts, css
    • src/pages (23 files, 2,156 lines)
      Purpose: Pages | File types: tsx, ts
    • src/api (18 files, 1,489 lines)  
      Purpose: Api | File types: ts
    • src/utils (12 files, 892 lines)
      Purpose: Utils | File types: ts

🏛️  ARCHITECTURE INSIGHTS
─────────────────────────
  Organization Score: 85.2%
  Complexity Level: Moderate
  Detected Patterns:
    • LayeredArchitecture
    • ComponentComposition
    • CustomHooks
```

## 🎯 Recommended Workflows

### Daily Development
```bash
sniff large && sniff imports
```

### Pre-commit Hook
```bash
sniff types && sniff env
```

### Performance Testing
```bash
sniff perf && sniff memory
```

### Pre-deployment (Complete Pipeline)
```bash
sniff deploy  # Runs comprehensive validation: env, types, large files, imports, bundle
```

### Individual Checks
```bash
sniff menu           # Interactive command selection
sniff large          # Check file sizes
sniff types          # TypeScript quality
sniff imports        # Unused imports
sniff bundle         # Bundle analysis
sniff perf           # Performance audit
sniff memory         # Memory leak detection
sniff env            # Environment validation
sniff context        # Project structure analysis
```

## 🚦 Exit Codes

- **0**: All checks passed (Success)
- **1**: General error
- **2**: Validation failed (issues found that need attention)
- **3**: Threshold exceeded (critical issues found)
- **4**: Configuration error

## 🔄 CI/CD Integration

### GitHub Actions

Add this workflow to `.github/workflows/sniff-check.yml`:

```yaml
name: Code Quality Check

on:
  pull_request:
    branches: [ main, develop ]
  push:
    branches: [ main, develop ]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Install sniff-check
      run: npx sniff-check@latest --version || npm install -g sniff-check
    
    - name: Run code quality checks
      run: |
        echo "🔍 Running sniff-check quality analysis..."
        sniff --json large > large-files.json
        sniff --json types > type-issues.json
        sniff --json imports > unused-imports.json
        
        # Show results in human-readable format
        echo "📊 Large Files Report:"
        sniff large
        
        echo "📝 TypeScript Quality:"
        sniff types
        
        echo "🚫 Unused Imports:"
        sniff imports
        
        echo "🏗️ Project Context:"
        sniff context
    
    - name: Upload reports as artifacts
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: sniff-check-reports
        path: |
          large-files.json
          type-issues.json
          unused-imports.json
        retention-days: 30
    
    - name: Comment PR with results
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v7
      with:
        script: |
          const fs = require('fs');
          try {
            const largeFiles = JSON.parse(fs.readFileSync('large-files.json', 'utf8'));
            const typeIssues = JSON.parse(fs.readFileSync('type-issues.json', 'utf8'));
            
            let comment = '## 🔍 Code Quality Report\n\n';
            comment += `**Large Files:** ${largeFiles.summary.large_files_found} found\n`;
            comment += `**Type Issues:** ${typeIssues.summary.total_issues} found\n`;
            comment += `**Type Coverage:** ${typeIssues.summary.type_coverage_score.toFixed(1)}%\n\n`;
            
            if (largeFiles.summary.critical > 0) {
              comment += '🚨 **Critical:** Files over 400 lines found - immediate refactoring needed!\n';
            }
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });
          } catch (error) {
            console.log('Could not create PR comment:', error);
          }

  performance-check:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Build project
      run: npm run build
    
    - name: Install sniff-check
      run: npm install -g sniff-check
    
    - name: Run performance analysis
      run: |
        sniff --json bundle > bundle-analysis.json
        sniff --json perf > performance-audit.json || echo "Performance audit skipped (no dev server)"
        sniff --json memory > memory-check.json
        
        # Show bundle analysis
        echo "📦 Bundle Analysis:"
        sniff bundle
    
    - name: Upload performance reports
      uses: actions/upload-artifact@v4
      with:
        name: performance-reports
        path: |
          bundle-analysis.json
          performance-audit.json
          memory-check.json
```

### Pre-deployment Pipeline

For production deployments:

```yaml
name: Pre-deployment Check

on:
  push:
    branches: [ main ]

jobs:
  deploy-check:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Install sniff-check
      run: npm install -g sniff-check
    
    - name: Run complete deployment validation
      run: |
        echo "🚀 Running complete pre-deployment validation..."
        sniff deploy
        
        # Additional context analysis
        echo "📊 Final project analysis:"
        sniff --json context > final-context.json
    
    - name: Block deployment on critical issues
      run: |
        # This will fail the workflow if critical issues are found
        if sniff --quiet types | grep -q "CRITICAL"; then
          echo "❌ Critical type issues found - blocking deployment"
          exit 1
        fi
        
        if sniff --quiet large | grep -q "CRITICAL"; then
          echo "❌ Critical large files found - blocking deployment"
          exit 1
        fi
        
        echo "✅ All checks passed - ready for deployment"
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Run `cargo test` and `cargo fmt`
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 💖 Support

If you find **sniff-check** helpful, consider supporting its development:

[![Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/rizkyviali)

Your support helps maintain and improve this tool for the entire TypeScript/Next.js community! ☕

## 🙏 Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [rayon](https://github.com/rayon-rs/rayon) - Parallel processing
- [regex](https://github.com/rust-lang/regex) - Pattern matching
- [colored](https://github.com/mackwic/colored) - Terminal colors

---

**sniff-check**: The definitive code quality tool for TypeScript/Next.js projects that developers actually want to use. 🎯