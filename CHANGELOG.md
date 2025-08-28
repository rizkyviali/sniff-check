# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-28

### ✨ New Features

#### 🧩 Component Analysis Command
- **New `sniff components` command** - Analyze and split large React/Vue/Angular components
- **Multi-framework support** - Detects React, Vue, Angular, and Svelte components
- **Intelligent complexity scoring** - Analyzes hooks, props, state, nesting, and logic complexity  
- **Smart refactoring suggestions** - Provides specific guidance for component splitting
- **Extractable parts detection** - Identifies custom hooks, utility functions, and sub-components
- **Framework-aware recommendations** - Tailored advice for each framework's best practices

#### 🚀 Enhanced Command System
- **Improved interactive menu** - Added components command to Code Quality section
- **Configurable thresholds** - Components command respects large files config for line limits
- **Detailed analysis reports** - Shows component type, complexity score, and specific issues

### 🔧 Infrastructure Improvements

#### 🎯 Smart Auto-Detection
- **Performance command** - Auto-detects running dev servers instead of hardcoded ports
- **Bundle command** - Framework-specific size recommendations (Next.js: 3MB, React: 2MB, Angular: 4MB, Svelte: 1MB)
- **Memory command** - Dynamic thresholds based on actual system memory (5% warning, 15% critical)

#### ⚙️ Configuration Enhancements  
- **Configurable line limits** - Large files command now uses `sniff.toml` thresholds
- **Dynamic severity labels** - File analysis displays use custom config values
- **Framework-specific limits** - Bundle analysis adapts to detected framework type

### 📊 Analysis Improvements
- **Cross-platform memory detection** - Linux, macOS, and Windows system memory detection
- **Smart Node.js recommendations** - Memory limits based on available system RAM
- **Framework-aware bundle analysis** - Different optimization strategies per framework

## [0.1.11] - 2025-08-27

### 🏗️ Code Architecture: Imports Module Refactoring

#### 📦 Modular Structure
- **Refactored imports.rs into modular package** - Extracted monolithic 600+ line imports.rs into dedicated imports_analyzer module
- **Separation of Concerns** - Organized code into specialized files: parser.rs, validation.rs, reporter.rs, types.rs, resolver.rs
- **Improved Maintainability** - Each module now handles specific aspects of import analysis
- **Enhanced Testability** - Modular structure enables focused unit testing of individual components

#### 🔧 Technical Improvements
- **parser.rs** - Dedicated import statement parsing logic
- **validation.rs** - Import usage validation and unused detection
- **reporter.rs** - Report generation and formatting
- **types.rs** - Shared data structures and type definitions
- **resolver.rs** - Import path resolution and broken import detection
- **mod.rs** - Public API and module coordination

#### 🎯 Developer Benefits
- **Easier Feature Development** - New import analysis features can be added to specific modules
- **Better Code Navigation** - Related functionality is logically grouped
- **Reduced Complexity** - Each file focuses on a single responsibility
- **Future-Proof Architecture** - Modular design supports future enhancements

### Impact
This refactoring maintains 100% backward compatibility while significantly improving code organization and maintainability. The imports command functionality remains identical for users.

## [0.1.10] - 2025-08-26

### ✨ Enhanced User Experience: Progress Feedback

#### 🔄 New Progress Feedback System
- **Real-time Progress Updates** - All sniff commands now provide clear, informative progress messages
- **Step-by-step Feedback** - Users see exactly what the tool is doing during analysis
- **Completion Indicators** - Clear confirmation when each analysis phase completes
- **Visual Improvements** - Emoji indicators and colored output for better readability

#### 📊 Command-Specific Progress Enhancements
- **large**: File scanning and analysis progress with file counts
- **deploy**: 5-step deployment pipeline with completion indicators (1/5, 2/5, etc.)
- **types**: TypeScript analysis progress with completion confirmation
- **memory**: Memory leak scanning with detailed progress steps  
- **env**: Environment validation progress for files and variables
- **context**: Comprehensive project analysis progress tracking
- **bundle**: Bundle analysis with build scanning progress

#### 🎯 User Experience Benefits
- **Transparency**: Users know the tool is working, not frozen
- **Confidence**: Clear feedback builds trust in long-running operations
- **Professional Feel**: Polished progress messages improve perceived quality
- **Debugging**: Progress messages help identify slow operations

#### 🔧 Technical Implementation
- Text-based progress approach avoids complex progress bar type issues
- Respects `--quiet` flag to suppress output when needed
- Cleaned up unused progress bar imports across all command files
- Maintains backward compatibility with all existing functionality

### Impact
This release significantly improves the developer experience by providing clear feedback during potentially long-running analysis operations. Users now see exactly what the tool is doing and when each step completes.

## [0.1.9] - 2025-08-26

### 🚀 MAJOR: Comprehensive Unused Import Detection Fix

This release represents a **revolutionary improvement** to import analysis, addressing the systematic false positive issues that were causing 100% incorrect results.

#### 🐛 Critical Fixes
- **React Hooks Detection**: Fixed detection of `useState`, `useEffect`, `useCallback`, and all React hooks in destructuring patterns
- **TypeScript Type Usage**: Now correctly identifies type usage in interfaces, type annotations, and generic constraints
- **JSX Component Detection**: Properly detects React component usage in JSX tags
- **Complex Type Patterns**: Fixed detection of types used in intersection types, union types, and complex generics
- **Function Parameter Types**: Now detects type usage in function signatures and parameter annotations

#### 🧠 Enhanced Analysis Patterns
- **Multi-pattern Detection**: Uses 7 different regex patterns for comprehensive usage detection
- **Type Annotation Scanning**: Detects usage in `: Type`, `<Type>`, `extends Type`, and `implements Type` patterns  
- **React Hook Patterns**: Specialized detection for `const [state, setState] = useState()` patterns
- **Generic Type Extraction**: Handles complex generics like `Array<User>`, `Promise<Result<Data>>`
- **Built-in Type Filtering**: Excludes TypeScript built-in types from false positive detection

#### 📊 Dramatic Results Improvement
- **Before**: ~100% false positive rate (752/752 imports incorrectly flagged)
- **After**: Near-zero false positives - only legitimately unused imports detected
- **Comprehensive Coverage**: React hooks, TypeScript types, JSX components, function parameters all properly detected

#### 🔬 Technical Implementation
- Added `extract_type_identifiers()` function for complex type parsing
- Implemented `is_typescript_builtin_type()` to filter built-in types  
- Enhanced regex patterns with proper word boundary detection
- Multi-pass analysis: import collection → comprehensive usage detection → accurate reporting

#### 🎯 Real-world Impact
This fix resolves the critical usability issue where the tool was essentially unusable due to overwhelming false positives. Now developers can trust the results and safely use automated import cleanup.

## [0.1.8] - 2025-08-26

### 🐛 Critical Bug Fix
- **TypeScript Inline Type Import Parsing** - Fixed critical parsing bug where inline type imports like `import { type NextRequest, NextResponse }` were incorrectly analyzed, causing false positives where "type" was reported as unused instead of the actual type name

### Technical Details
- Enhanced named import parser to handle inline `type` modifiers correctly
- Now properly extracts type names after the `type` keyword in mixed import statements
- Maintains backward compatibility with all existing import patterns
- Zero breaking changes - purely a bug fix release

### Impact
- Eliminates false positive "Unused: type" warnings for inline type imports
- Improves developer experience by providing accurate unused import detection
- Particularly beneficial for Next.js projects using server components with type imports

## [0.1.7] - 2025-08-26

#### 🐛 Critical Bug Fixes
- **Import Regex Parsing Fix** - Resolved issue where import statements with trailing comments weren't detected
- **TypeScript Type Imports** - Fixed parsing of `import type { A, B }` statements that were being truncated
- **Test Infrastructure** - Fixed CommandRunner to use release binary correctly, ensuring reliable CI/CD
- **Exit Code Consistency** - Standardized exit codes across all commands for better CI/CD integration

#### ⚡ Performance & Infrastructure
- **Progress Tracking Foundation** - Added infrastructure for progress indicators on large projects (>50 files)
- **Sequential Processing** - Large projects now use progress-tracked sequential processing
- **Parallel Processing** - Maintained high-speed parallel processing for smaller projects
- **JSON Output Enhancement** - Broken imports now properly included in JSON responses

#### 🧹 Code Quality & Maintenance  
- **Critical Clippy Fixes** - Resolved bool assertion warnings and format string issues
- **Test Suite Reliability** - All tests now pass consistently, ensuring stable releases
- **Documentation Updates** - Enhanced README and CHANGELOG for npm publication
- **Version Consistency** - Synchronized versions across all configuration files

#### 🎯 Enhanced User Experience
- **Better Error Messages** - Improved guidance with smart suggestions for broken imports
- **Visual Improvements** - Enhanced terminal output with better formatting and colors
- **npm Publication Ready** - Optimized package metadata and installation instructions

### Technical Improvements
- **Smart Import Detection** - Now correctly handles comments at end of import lines
- **Path Resolution** - Enhanced relative import path checking and validation
- **Suggestion Engine** - Intelligent recommendations for fixing broken imports
- **CI/CD Integration** - Improved JSON output format for automated builds

### 🔧 Migration Notes
- All existing functionality remains unchanged (100% backward compatible)
- Enhanced import detection may find previously missed issues (this is a good thing!)
- JSON output now includes `broken_imports` field alongside existing `unused_imports`
- Exit codes are now consistent: 0=success, 2=validation issues found

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