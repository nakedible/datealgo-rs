# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/nakedible/datealgo-rs/compare/v0.2.0...v0.2.1) - 2023-12-20

### Added
- Improve performance of `rd_to_weekday`

### Other
- Fix a couple errors in benchmark inputs
- Update acknowledgements section
- Tweak formatting of `rd_to_weekday` algo
- Add a lot more checks to ci
- Make two links into hyperlinks
- Add couple clippy allows
- Reformat
- Remove old asm dumps and add new
- Implement new way to dump asm
- Update criterion to 0.5.1
- Update iai to 0.9.0 and fix tests
- Add old `rd_to_weekday` to compare benchmarks
- Run benchmarks on pull requests again
- fix typo in documentation.
- Update benchmark disclaimer
- Update benchmark snapshot in docs
- More reliable benchmarks by array input
- Create utility module for benches

## [0.2.0](https://github.com/nakedible/datealgo-rs/compare/v0.1.1...v0.2.0) - 2023-10-31

### Added
- Add `next_date` and `prev_date` functions
- Add ISO week calculations

### Other
- Add forbid unsafe, just to declare intent
- Fix typos in documentation.
- Update asm dump
- Add old `secs_to_dhms` as compare bench
- Improve performace of `secs_to_dhms`

## [0.1.1](https://github.com/nakedible/datealgo-rs/compare/v0.1.0...v0.1.1) - 2023-09-10

### Fixed
- Change month and weekday constants to `u8`

### Other
- Update asm dump

## [0.1.0](https://github.com/nakedible/datealgo-rs/compare/v0.0.6...v0.1.0) - 2023-09-08

### Added
- Tiny optimization to `is_leap_year`

## [0.0.6](https://github.com/nakedible/datealgo-rs/compare/v0.0.5...v0.0.6) - 2023-09-02

### Added
- Optimize `is_leap_year` and `days_in_month`

### Other
- Add also generated asm to track evolution
- Add asm dump script using godbolt
- Add many alternatives to compare benchmark
- Add positive `is_leap_year` tests
- Slight cleanups and comments to clarify
- Update benchmarks
- Add script for dumping performance results
- Change compare benchmark to use rand values
- Change basic benchmark to use random values
- Check more with quickcheck

## [0.0.5](https://github.com/nakedible/datealgo-rs/compare/v0.0.4...v0.0.5) - 2023-08-30

### Added
- Make conversion to `SystemTime` be fallible

### Other
- Add first version of contributing guide
- Set default criterion settings for more reliable benchmarks

## [0.0.4](https://github.com/nakedible/datealgo-rs/compare/v0.0.3...v0.0.4) - 2023-08-29

### Added
- Use `u8` where ever possible

### Other
- Remove criterion run from CI as it is useless in CI
- Add iai benchmarking as well
- Remove outdated commented benches
- Change names of benches to avoid conflicts
- Small optimisations of `rd_to_date`.
- Fix typo in Unix epoch

## [0.0.3](https://github.com/nakedible/datealgo-rs/compare/v0.0.2...v0.0.3) - 2023-05-20

### Other
- Fix changelog link
- Add acknowledments section
- Only run basic bench in actions
- Reorganize benchmarks
- Allow benchmark runner to modify pull
- Add automatic pull request criterion bench
- Remove version from README
- Remove warning from README

## [0.0.2](https://github.com/nakedible/datealgo-rs/compare/v0.0.1...v0.0.2) - 2023-05-20

### Added
- Remove warning notice, ready for use

### Other
- Add CODEOWNERS
- Fix release plz branch name
- Remove publish workflow
- Add release-plz workflow
- Change docs to separate errors
- Switch to canonical way to do no_std
- Improve quickchecks
- Add minimal quickcheck test
- Move tests to external dir
- Add some missing tests
- Small docs updates
- Reorganize benchmark lines
- Make docs and crates badges links
- Some updates
