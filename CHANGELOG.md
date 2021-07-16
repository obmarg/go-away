# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project intends to inhere to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html), but has not yet reached 1.0 so
all APIs might be changed.

## Unreleased - xxxx-xx-xx

### Bug Fixes

- Improved some of the clearly wrong formatting on the output code

## v0.3.0 - 2021-07-02

### Breaking Changes

- Enum variant constants no longer contain underscores.
- `chrono::DateTime<Utc>` is now translated to `time.Time` rather than `String`

## v0.2.0 - 2021-06-06

### Breaking Changes

- Enums variant constants now prefixed with their type name to avoid clashes.

### New Features

- Internally tagged enums are now supported
- `&str` fields are now supported
- `go-away` now deduplicates types so if a given type appears in more than one
  place it should only result in one go type being output.
- The go output is now tested, so should work better

### Bug Fixes

- Go union type output is now more likely to compile (it didn't at all prior to
  this)
- Single field structs are no longer considered newtypes
- Go marshalling now works (it had bugs previously)

### Changes

- Messed with the order of output a bit

## v0.1.1 - 2021-05-26

### Changes

- Fixed some documentation & CHANGELOG links

## v0.1.0 - 2021-05-26

- Initial release
