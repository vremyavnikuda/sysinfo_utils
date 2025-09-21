# Changelog

All notable changes to the `sysinfo_utils` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Pre-release: 2025/09/22]

### Added
- Async GPU info retrieval with provider fallback
- Logging for monitoring thread lifecycle

### Changed
- Replaced generic thread imports with specific ones

### Fixed
- Vendor matching in `update_gpu` using discriminant
- Fallback behavior when no provider registered

### Removed
- Redundant comments in test modules