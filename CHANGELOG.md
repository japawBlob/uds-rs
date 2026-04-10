# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Rust Semantic Versioning](https://doc.rust-lang.org/cargo/reference/semver.html).

## [Unreleased]

## [0.3.0] - 2026-04-10
### Added
- Added `UdsSocketOptions::vw()` helper to build VW-specific ISO-TP options while keeping the main API generic.

### Changed
- **BREAKING**:`UdsClient::new` requires an explicit `UdsSocketOptions`

### Removed
- **BREAKING**: `new_vw` constructor; use `new` with `UdsSocketOptions::vw()` instead

## [0.2.1] - 2026-03-14
### Changed
- Generate readme from docs in code
- Update dependencies

## [0.2.0] - 2026-03-13
### Added
- Session control
- Time-out for content not ready
- UDS over CAN client with padding

### Changed
- Make all response fields `pub`
- Bumping dependencies, fix clippy and re-format code

## [0.1.0] - 2024-01-09
### Added
- Initial release with core features:
  - Simple set-up including CAN bus socket with given sender and receiver IDs
  - Asynchronous UDS over CAN client with support for one request/response at a time
