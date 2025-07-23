# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Correctly handle Lighme Logbook roll load/unload dates with no associated time value

## [0.1.0] - 2025-07-14

### Added

- This is a full rewrite of the [filmrolls gem](https://rubygems.org/gems/filmrolls)
- Support for [Lightme Logbook iOS app](https://apps.apple.com/us/app/lightme-logbook/id1544518308) JSON metadata

### Changed

- The author metadata is TOML-based instead of YAML-based

## Removed

- Unlike the ruby-based implementation, IPTC metadata is not supported

[Unreleased]: https://github.com/urdh/filmrolls-rs/commit/v0.1.0...HEAD
[0.1.0]: https://github.com/urdh/filmrolls-rs/releases/tag/v0.1.0
