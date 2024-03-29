# Changelog

## Table of Contents

- [Version 0.3](#version-03)
- [Version 0.2](#version-02)
    - [v0.2.1](#v021)
    - [v0.2.0](#v020)

## Version 0.3

### v0.3.5
Fixed a bug preventing the mode function from running. ([See #4](https://github.com/maxblee/clipivot/issues/4).)

#### Added
- Changes to Cargo.toml to point repository here. ([#4](https://github.com/maxblee/clipivot/issues/4).)

#### Fixed
- The mode function wasn't running due to an improper configuration. This should be fixed. ([#4](https://github.com/maxblee/clipivot/issues/4).)

### v0.3.4
Changed handling date formatting to remove lifetime requirements.

#### Changed
`clipivot::parsing::set_date_format` now takes in a String instead
of a `&'static str`.

### v0.3.3
Changes to date formatting.

#### Changed
- Changed handling of date formatting so you can modify date formatting multiple times.
- Fixed a bug in date parsing that prevented dates from being formatted without time information.

### v0.3.2
Changes `Aggregator::aggregate` to take a mutable reference
for writing results.

#### Changes
Changes `Aggregator::aggregate` to take a mutable reference
for writing results.

### v0.3.1
Changes `Aggregator::aggregate` to take a mutable reference to a `csv::Reader`.

#### Changed
- `Aggregator::aggregate` takes a mutable reference to a `csv::Reader`

### v0.3.0
For people using the command-line tool, there aren't going to be any
differences you'll notice. The only changes in this version were to core code in Rust.

All of these changes were slight re-designs so I could port the code I wrote into a WebAssembly package. However, one of the changes I made was a breaking change, so I'm upping the minor version.

#### Added
- Added a function (just in the core Rust) allowing you to export the aggregations as a vector
of vectors of strings.

#### Changed
- Revised the documentation
- Changed the function to write results from the aggregations to make it work for anything that implements `std::io::Write`, instead of only working on `std::io:stdout`.

#### Removed
- Removed `impl std::error::Error` because the only method I implemented
is now deprecated.

## Version 0.2

### v0.2.1
#### Added
- Added release artifacts (zipped files containing the `README`, licenses and binary release)

#### Changed
- Merged from Travis CI to GitHub Actions

### v0.2.0

This version primarily added support for sorting records and for viewing the minimum and maximum values. [See #2 for details](https://github.com/maxblee/clipivot/issues/2).

#### Added
- Allowed for sorting the output of the columns and rows (by default, the columns sort in ascending order, while the rows appear in index order)
- Added the `minmax` function to provide an easy way to see both the minimum and maximum values.
- Added MIT License

#### Changed
- Rewrote/revised documentation
- Refactored most of the code base
- Replaced `enum`-based text parsing with generic typing

#### Removed
- Removed support for the `dtparse` library.
- Removed YAML dependency for `clap`.