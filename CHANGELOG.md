# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Fix appveyor CI issue with directory names.
- Holding CMD in search results now correctly shows either tags or URL based on users' settings.

## [0.15.4] - 2019-06-17
### Added
- Add option to either show TAGs or URLs in search results.
- Add a combo modifier for search result to copy URL to clipboard.
### Fixed
- Fix multiple issues related to release of Alfred 4
- `pcheck` should now force a network call regardless of when last update check was done.
- Fix: deleting a bookmark was not working.

## [0.14.9] - 2019-02-13
### Added
- Add settings for notifying if page is already bookmarked.

## [0.14.8] - 2019-02-13
### Fixed
- Workaround for Firefox ([Fixes #25](https://github.com/spamwax/alfred-pinboard-rs/issues/25))

## [0.14.7] - 2019-01-30
### Added
- Support [Brave Browser](brave.com)

## [0.14.6] - 2019-01-22
### Added
- Minor improvements

## [0.14.5] - 2019-01-15
### Added
- Show whether current page is already bookmarked.

## [0.14.4] - 2018-11-22
### Fixed
- Fixes issue [#21](https://github.com/spamwax/alfred-pinboard-rs/issues/21)

## [0.14.1 - 0.14.3] - 2018-08-27 - 2018-10-31
### Fixed
- Re-enable auto cache update
- Using `;` to add description was broken
- Recompile binary to fix an upstream bug

### Added
- Add Opera support

## [0.14.0] - 2018-06-04
### Added
- Workflow can notify and auto update itself.

## [0.13.3] - 2018-05-29
### Fixed
- Fixes issue [#7](https://github.com/spamwax/alfred-pinboard-rs/issues/7)
