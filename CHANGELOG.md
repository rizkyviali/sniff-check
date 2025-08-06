# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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