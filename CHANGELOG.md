# CHANGELOG

All notable changes to the Phlow project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.44] - 2025-08-04

### Added

#### 🆕 Phlow Modules (.phlow files)
- **NEW FEATURE**: Support for creating modules using pure Phlow syntax without Rust compilation
- Added schema validation for Phlow modules with `with`, `input`, `output`, and `steps` sections
- Added support for `setup`, `main`, and `payload` variables in Phlow modules
- Added automatic module type detection (Binary vs Script)
- Added comprehensive examples for route handling, data transformation, and authentication modules

#### 📄 New Example Project: `import_phlow`
- Added complete HTTP API example demonstrating Phlow module usage
- Created `main.phlow` with REST API for author management (GET, POST, DELETE)
- Added `route.phlow` - reusable routing module with schema validation
- Added `return.phlow` - utility module for response handling
- Included comprehensive test suite with 4 test scenarios

#### 🎯 Enhanced Include System
- Added support for arguments in `!include` directive
- Introduced `!arg` directive for compile-time argument substitution
- Enhanced template-like behavior for reusable Phlow components
- Added error handling for missing required arguments

#### 📚 Documentation Expansion
- Created comprehensive documentation for Phlow modules in `site/docs/packages-and-modules/phlow-modules.md`
- Updated module documentation in `site/docs/phlow-structure/modules.md` with Phlow module examples
- Enhanced `packages-and-modules/introduction.md` with examples showing both module types
- Added comparison tables between Phlow and Rust modules

### Enhanced

#### 🔧 Runtime Improvements
- Enhanced pipeline execution with better error handling and logging
- Improved context management with encapsulated property access
- Added pipeline ID tracking for better debugging
- Enhanced step execution with improved error reporting
- Better handling of undefined runtime responses

#### 🧪 Testing Framework
- Improved test execution with better error messages
- Enhanced test runner with Settings parameter support
- Added comprehensive tests for the new route system
- Better payload handling in test scenarios

#### 🔄 Module Loading System
- Refactored module loading to support both binary and script modules
- Enhanced local module detection and loading
- Improved module path resolution for .phlow files
- Added debug logging for module loading operations
- Better error handling for module not found scenarios

#### 📝 Script System
- Enhanced PHS (Phlow Script) with access to `setup` variable
- Improved script evaluation with better context handling
- Added support for compile-time argument processing
- Enhanced script loading with better error reporting

### Changed

#### 📖 Documentation Updates
- Updated all code examples to use `.phlow` syntax highlighting instead of YAML
- Enhanced directive documentation with argument support examples
- Updated philosophy section to mention Phlow files instead of YAML
- Improved structure documentation with better examples

#### 🏗️ Architecture Improvements
- Refactored loader system to handle different module types
- Enhanced context system with getter methods for better encapsulation
- Improved pipeline handling with better step tracking
- Simplified module execution flow

#### 🔧 Configuration Changes
- Updated environment variable from `PHLOW_YAML_SHOW` to `PHLOW_SCRIPT_SHOW`
- Enhanced module configuration with better local path support
- Improved VS Code launch configuration for debugging

### Fixed

#### 🐛 Bug Fixes
- Fixed module loading for local .phlow files
- Improved error handling in script execution
- Better handling of empty payloads in return operations
- Fixed pipeline execution with proper step sequencing
- Improved module path formatting and extension handling

#### 🔍 Error Handling
- Enhanced error logging with proper log levels
- Better error messages for module loading failures
- Improved debugging output for transformation errors
- Added proper error propagation in pipeline execution

### Dependencies

#### 📦 Updated Dependencies
- Updated Tokio from 1.47.0 to 1.47.1
- Bumped workspace version from 0.0.43 to 0.0.44
- Updated all workspace packages to version 0.0.44

### Development

#### 🛠️ Development Tools
- Added new VS Code launch configuration for running with current file
- Enhanced debugging capabilities with better logging
- Improved development workflow with file watching support

#### 🧹 Code Cleanup
- Removed obsolete test files and unused code
- Cleaned up module implementations (echo, postgres, rpc)
- Simplified loader logic and improved maintainability
- Removed unnecessary debug logs and improved code structure

### Documentation

#### 📚 Major Documentation Improvements
- **Comprehensive Phlow Module Guide**: New 400+ line documentation covering creation, usage, and best practices
- **Enhanced Module Documentation**: Updated existing module docs with Phlow module examples
- **Improved Examples**: All documentation examples now use proper Phlow syntax
- **Better Navigation**: Enhanced documentation structure for easier discovery

#### 🎨 Documentation Features
- Added comparison tables between Phlow and Rust modules
- Included practical examples for different use cases
- Enhanced code snippets with proper syntax highlighting
- Added troubleshooting sections and best practices

### Infrastructure

#### ⚠️ Development Status
- Added development warning to README indicating active development status
- Enhanced project metadata and documentation structure

---

## Summary

This release introduces **Phlow Modules** - a revolutionary new way to create reusable components using pure Phlow syntax without requiring Rust knowledge. This feature democratizes module creation and significantly accelerates development workflows.

Key highlights:
- 🆕 **Phlow Modules (.phlow)** - Create modules without compilation
- 🎯 **Enhanced Include System** - Template-like includes with arguments  
- 📄 **Complete Example Project** - Real-world HTTP API demonstration
- 📚 **Comprehensive Documentation** - 400+ lines of new documentation
- 🔧 **Runtime Improvements** - Better error handling and debugging
- 🧪 **Enhanced Testing** - Improved test framework and examples

This release represents a significant evolution in the Phlow ecosystem, making it more accessible while maintaining the performance and reliability that makes Phlow powerful.
