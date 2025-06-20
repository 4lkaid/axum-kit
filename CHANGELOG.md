# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.6] - 2025-06-17

### Changed

- Bump redis to v0.32.

## [0.6.5] - 2025-05-12

### Changed

- Bump redis to v0.31.

## [0.6.4] - 2025-04-24

### Changed

- Bump redis to v0.30.

### Removed

- Removed `bb8-redis` dependency (replaced with `bb8` feature from `redis` crate).

## [0.6.3] - 2025-02-25

### Added

- Added `bb8-redis` as an optional dependency to support Redis connection pooling.

### Changed

- Updated `redis` dependency to remove the `tokio-comp` feature, as it is no longer needed.
- Refactored Redis connection management to use `bb8::Pool` and `RedisConnectionManager`.
- Replaced `OnceLock<redis::Client>` with `OnceLock<bb8::Pool<RedisConnectionManager>>`.
- Updated `init` and `conn` functions to initialize and retrieve connections from the connection pool.

## [0.6.2] - 2025-02-17

### Changed

- Bump redis to v0.29.

## [0.6.1] - 2025-01-22

### Changed

- Bump validator to v0.20.
- Bump MSRV to 1.81.

## [0.6.0] - 2025-01-19

### Added

- Added `env-filter` feature to `tracing-subscriber`.

### Changed

- Replaced `with_max_level` with `EnvFilter`.
- **Breaking**: `Application::default` no longer takes `router`. Use `with_router`.
- **Breaking**: `trace_body` now returns `Either<TraceBodyLayer, Identity>`.

## [0.5.2] - 2025-01-16

### Added

- Added `DEFAULT_ERROR_LEVEL` and `DEFAULT_MESSAGE_LEVEL` constants for unified log level configuration.
- Added `event_dynamic_lvl!` macro to support dynamic log level-based logging.

### Changed

- Moved the definition of `DEFAULT_MESSAGE_LEVEL` from `trace.rs` to `mod.rs` for centralized management.
- Added a `level` field to `TraceBodyLayer` and `TraceBody` to support custom log levels.
- Modified the `collect_and_log` function to support dynamic log level-based logging.

### Fixed

- Fixed the implementation of the `trace_body` function to return `TraceBodyLayer::default()`.

## [0.5.1] - 2025-01-08

### Added

- CHANGELOG.md.

### Changed

- Bump redis to v0.28.

[unreleased]: https://github.com/4lkaid/axum-kit/compare/v0.6.6...HEAD
[0.6.6]: https://github.com/4lkaid/axum-kit/compare/v0.6.5...v0.6.6
[0.6.5]: https://github.com/4lkaid/axum-kit/compare/v0.6.4...v0.6.5
[0.6.4]: https://github.com/4lkaid/axum-kit/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/4lkaid/axum-kit/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/4lkaid/axum-kit/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/4lkaid/axum-kit/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/4lkaid/axum-kit/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/4lkaid/axum-kit/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/4lkaid/axum-kit/compare/v0.5.0...v0.5.1
