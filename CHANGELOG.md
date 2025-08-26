# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - 2025-08-26

### 🎯 New Feature: Broken Imports Detection

#### Added
- **💥 Broken Imports Detection** - Enhanced `sniff imports` command now detects broken and invalid imports
- **🔍 File Not Found Detection** - Identifies imports referencing non-existent files (perfect for refactoring scenarios)
- **📦 Module Installation Check** - Detects imports from uninstalled npm packages
- **💡 Smart Suggestions** - Provides intelligent fix suggestions for broken imports
- **🎯 Refactoring Support** - Perfect for detecting issues after moving/renaming files

#### Enhanced Features
- **🔧 Comprehensive Import Analysis** - Single command now checks both unused AND broken imports
- **📊 Unified Reporting** - Shows unused and broken imports together with clear error messages
- **🎨 Improved Visual Feedback** - Better colors and icons to distinguish between different issue types
- **📈 Enhanced Summary** - Updated summary includes both unused and broken import counts

#### Perfect for Refactoring Workflows
- **File Moves/Renames** - Instantly identifies all imports that need updating after file changes
- **Package Management** - Quickly spot missing dependencies that need installation
- **Code Cleanup** - Remove unused imports and fix broken ones in one go
- **Development Workflow** - Catch import issues before they cause compilation errors

### Technical Improvements
- **⚡ Intelligent Path Resolution** - Handles relative imports with proper directory traversal
- **🧠 Smart Package Detection** - Correctly identifies scoped packages (@types/node, @scope/package)
- **📁 Extension Handling** - Tries common JavaScript/TypeScript file extensions automatically
- **🔍 Similar File Suggestions** - When files aren't found, suggests similar files in nearby directories

### Usage Examples
```bash
# Detect both unused and broken imports
sniff imports

# JSON output for CI/CD integration
sniff imports --json

# Example output shows:
# 💥 File not found: ./old-component
# 💡 Suggestion: ./components/new-component
# 💥 Module not installed: lodash
# 💡 Run: npm install lodash
```

## [0.1.5] - 2025-08-18

### 🚀 Major Performance & Architecture Improvements

#### Added
- **⚡ Performance Optimization System** - New `OptimizedFileWalker` with smart filtering and parallel processing
- **📊 Performance Monitoring** - Built-in performance tracking with `SNIFF_PERF_DEBUG=1` environment variable
- **🧪 Comprehensive Test Framework** - Shared test utilities with `TestProject`, `SampleFiles`, and `CommandRunner`
- **📋 Unified JSON Output Format** - Standardized responses with timestamps, version info, and metadata
- **🎯 Common CLI Patterns** - Reusable argument structures and output utilities
- **🔧 Centralized Error Handling** - Standardized error codes and reporting across all commands
- **🗂️ Shared Common Module** - Eliminated code duplication with centralized utilities

#### Performance Improvements
- **File Discovery**: Optimized with smart directory exclusion and depth limits
- **Parallel Processing**: Configurable thresholds (20+ files automatically use parallel processing)
- **Memory-Mapped Line Counting**: Large files (1MB+) use memory mapping for faster analysis
- **Smart File Filtering**: Excludes binary files, images, and common build artifacts automatically
- **Reduced Allocation**: Better data structures and caching reduce memory usage

#### Architecture Enhancements
- **📁 New `src/common/` Module**: Centralized shared utilities
  - `file_scanner.rs` - Unified file discovery
  - `regex_patterns.rs` - Shared regex compilation
  - `error_handler.rs` - Standardized error handling
  - `json_output.rs` - Unified JSON responses
  - `performance.rs` - Performance optimizations
  - `cli_args.rs` - Common CLI patterns
  - `output_utils.rs` - Standardized output

#### Quality Improvements
- **🧹 Code Deduplication**: Eliminated ~300 lines of duplicate code and ~15 duplicate functions
- **📝 Reduced Warnings**: Cleaned up unused imports and dead code
- **🔒 Type Safety**: Fixed compilation errors and improved type annotations
- **📈 Test Coverage**: Comprehensive integration tests for all commands

### Changed
- **JSON Output Structure**: Now includes `command`, `timestamp`, `version`, and performance metrics
- **Error Handling**: Standardized exit codes (0=success, 1=general error, 2=validation failed, 3=threshold exceeded)
- **CLI Output**: Consistent status messages and formatting across all commands
- **Performance**: File analysis improved from ~283ms to ~1.3ms for typical projects

### Developer Experience
- **🐛 Debug Mode**: Set `SNIFF_PERF_DEBUG=1` to see detailed performance breakdowns
- **📊 Rich Metrics**: JSON output includes analysis duration and processing statistics
- **🎨 Better Formatting**: Consistent colored output and status indicators
- **⚙️ Extensible**: New architecture makes adding features much easier

### Migration Notes
- All existing CLI commands and flags remain unchanged (100% backward compatible)
- JSON output structure enhanced but maintains compatibility with existing tooling
- Performance improvements are automatic - no configuration changes needed

## [0.1.4] - 2025-08-06

### Added
- Animated progress bars and spinners for better user experience during analysis
- Smart context-aware loop analysis to detect break conditions in `while(true)` loops
- Configuration options to disable specific memory leak pattern types
- Better exclusion of third-party directories (`node_modules`, `.next`, `dist`, etc.)

### Changed
- Downgraded `while(true)` pattern severity from Critical to Medium for more balanced reporting
- Improved pattern matching to significantly reduce false positives in memory analysis
- Enhanced memory leak detection with smarter filtering of legitimate code patterns
- Progress indicators now properly respect `--quiet` flag across all commands

### Fixed
- Memory analysis no longer flags legitimate patterns in `node_modules` and vendor code
- Progress bars now animate correctly during file scanning and analysis operations
- Context-aware detection prevents false alarms for loops with proper exit conditions

## [0.1.3] - 2025-08-06

### Fixed
- **Critical memory command crashes** - Resolved regex parsing errors that caused panics
- **Improved error handling** - Commands now properly exit with appropriate error codes
- **Enhanced pattern matching** - Removed unsupported regex features (lookahead/lookbehind)

### Added
- **Expanded file exclusions** - Added support for `coverage`, `.cache`, `pnpm-lock.yaml`, `bun.lockb`, `*.d.ts`
- **Enhanced Next.js support** - Added file type classifications for API routes, server components, client components, middleware, and custom hooks
- **Comprehensive environment variables** - Added `NEXTAUTH_SECRET`, `NEXTAUTH_URL`, `DATABASE_URL`, `VERCEL_URL` to defaults
- **Better configuration management** - Enhanced `sniff.toml` structure with modern development practices

### Changed
- **Reduced compiler warnings** - Cleaned up unused imports and dependencies
- **Updated binary distribution** - Properly compiled binary now included in package
- **Version consistency** - Synchronized version numbers across Cargo.toml and package.json

### Security
- All commands are now fully functional and production-ready

## [0.1.2] - 2025-08-05

### Added
- Initial TypeScript analysis with 'any' type detection
- Large file detection and refactoring suggestions
- Bundle analysis for optimization opportunities
- Memory leak pattern detection
- Environment variable validation
- Project context analysis
- Pre-deployment validation pipeline

### Features
- **Large Files Detection** - Find and refactor "smelly code" files over 100 lines
- **TypeScript Quality Check** - Comprehensive type analysis with coverage scoring
- **Unused Imports Detection** - Clean up dead imports automatically
- **Bundle Analysis** - Optimize build output and bundle sizes
- **Performance Auditing** - Lighthouse integration for performance testing
- **Memory Leak Detection** - Monitor Node.js memory usage patterns
- **Environment Validation** - Check required environment variables
- **Project Context Analysis** - Comprehensive project structure insights
- **Configuration System** - Project-specific settings via `sniff.toml`

## [0.1.0] - 2025-08-05

### Added
- Initial release of sniff-check
- Basic CLI structure with interactive menu
- Core file scanning functionality
- TypeScript/JavaScript file analysis
- Configuration system foundation

---

## Release Notes

### Memory Analysis Improvements (Latest)
The memory analysis system has been significantly enhanced to provide **actionable, precise feedback** instead of overwhelming false positives:

- **🚫 No more false alarms** from third-party libraries in `node_modules`
- **🧠 Smart detection** distinguishes between problematic and legitimate code patterns
- **⚙️ Configurable behavior** allows customization for different project needs
- **🎯 Context-aware analysis** reduces noise from React/framework patterns
- **📊 Better reporting** with cleaner JSON output for CI/CD integration

The tool now provides **developer-friendly insights** that teams can actually act upon, rather than noise that gets ignored.

### Upcoming Features
- Enhanced performance analysis with Core Web Vitals tracking
- Advanced bundle optimization recommendations  
- Integration with popular CI/CD platforms
- Custom rule configuration for team-specific standards
- Support for additional frameworks (Vue, Svelte, Angular)

---

For more detailed usage instructions and examples, see [README.md](./README.md).