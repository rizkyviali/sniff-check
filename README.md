# sniff-check

> Opinionated TypeScript/Next.js Development Toolkit

A comprehensive Rust CLI tool that enforces opinionated code quality standards for TypeScript/Next.js projects. Fast, reliable, and provides actionable feedback to developers.

## 🔥 Core Philosophy

- **Files over 100 lines are considered "smelly code"** and should be refactored
- **TypeScript 'any' usage is forbidden** - strict typing enforced
- **Clean separation** between utilities and components
- **Zero-tolerance** for unused imports and dead code
- **Comprehensive pre-deployment validation**

## 🚀 Installation

### Via npm (Recommended)
```bash
# Install globally
npm install -g sniff-check

# Or use with npx (no installation required)
npx sniff-check

# Or install locally in your project
npm install --save-dev sniff-check
```

### Via Cargo (Rust)
```bash
# Install from crates.io
cargo install sniff-check

# Or build from source
git clone https://github.com/rizkyviali/sniff-check
cd sniff-check
cargo build --release
```

### Requirements
- Node.js 14+ (for npm installation)
- Rust 1.70+ (for cargo installation or building from source)

## 📖 Usage

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
sniff large --threshold 150  # Custom threshold
```

Scans all TypeScript/JavaScript files and flags files over the threshold as "smelly code". Provides specific refactoring suggestions based on file type (component, service, API, etc.).

**Severity Levels:**
- **Warning** (100-200 lines): Needs attention
- **Error** (200-400 lines): Should be refactored
- **Critical** (400+ lines): Must be refactored immediately

#### 📝 TypeScript Quality Check
```bash
sniff types
```

Comprehensive TypeScript analysis:
- Detects 'any' type usage (CRITICAL)
- Finds missing return type annotations
- Identifies @ts-ignore/@ts-expect-error comments
- Calculates type coverage score (0-100%)

#### 🚫 Unused Imports Detection
```bash
sniff imports
```

Smart analysis of import statements:
- Detects unused default, named, and namespace imports
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
sniff large --json

# Quiet mode for CI environments
sniff large --quiet

# Custom configuration
sniff --config custom.toml large
```

## 🎯 Features

### ✅ Fully Implemented
- **Interactive Menu** - Beautiful terminal UI with categorized commands
- **Large Files Detection** - Find and refactor "smelly code" files
- **TypeScript Quality Check** - Comprehensive type analysis and scoring
- **Unused Imports Detection** - Clean up dead imports automatically
- **Bundle Analysis** - Optimize build output and bundle sizes
- **Performance Auditing** - Lighthouse integration for performance testing
- **Memory Leak Detection** - Monitor Node.js memory usage patterns
- **Environment Validation** - Check required environment variables
- **Project Context Analysis** - Comprehensive project structure and insights
- **Pre-deployment Pipeline** - Complete validation before deployment
- **Configuration System** - Project-specific settings and overrides

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
]
excluded_files = [
    "*.min.js",
    "*.bundle.js", 
    "package-lock.json",
    "yarn.lock",
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
]
check_security = true
allow_empty_values = false
env_files = [
    ".env",
    ".env.local",
    ".env.development", 
    ".env.production",
]
```

Use `sniff config init` to generate a default configuration file, or `sniff config show` to see your current settings.

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

- **0**: All checks passed
- **1**: Issues found that need attention
- **2**: Configuration error
- **3**: Build/project setup error

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
        sniff large --json > large-files.json
        sniff types --json > type-issues.json
        sniff imports --json > unused-imports.json
        
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
        sniff bundle --json > bundle-analysis.json
        sniff perf --json > performance-audit.json || echo "Performance audit skipped (no dev server)"
        sniff memory --json > memory-check.json
        
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
        sniff context --json > final-context.json
    
    - name: Block deployment on critical issues
      run: |
        # This will fail the workflow if critical issues are found
        if sniff types --quiet | grep -q "CRITICAL"; then
          echo "❌ Critical type issues found - blocking deployment"
          exit 1
        fi
        
        if sniff large --quiet | grep -q "CRITICAL"; then
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

## 🙏 Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [rayon](https://github.com/rayon-rs/rayon) - Parallel processing
- [regex](https://github.com/rust-lang/regex) - Pattern matching
- [colored](https://github.com/mackwic/colored) - Terminal colors

---

**sniff-check**: The definitive code quality tool for TypeScript/Next.js projects that developers actually want to use. 🎯