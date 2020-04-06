# Changelog

## Version 0.2.0

### Added
- Allowed for sorting the output of the columns and rows (by default, the columns sort in ascending order, while the rows appear in index order)
- Added the `minmax` function to provide an easy way to see both the minimum and maximum values.
- Added MIT License

### Changed
- Rewrote/revised documentation
- Refactored most of the code base
- Replaced `enum`-based text parsing with generic typing

### Removed
- Removed support for the `dtparse` library.
- Removed YAML dependency for `clap`.