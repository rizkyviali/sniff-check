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
```

## 🚦 Exit Codes

- **0**: All checks passed
- **1**: Issues found that need attention
- **2**: Configuration error
- **3**: Build/project setup error

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