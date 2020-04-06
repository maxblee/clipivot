# cli_testing_utils
A suite of utilities I've created for testing command-line interfaces

## Testing Command-line interfaces in Rust
This is a small experimental library I've created for testing command-line interfaces in Rust.

So small, in fact, that right now, it's just a single macro that makes it easier to run integration
tests on CLIs and have them work (i.e. pass) with Travis-CI.

As I create new command-line tools in Rust, this tool will probably grow a bit. Every time I'll add a feature
or make breaking changes to an existing feature, I'll upgrade the minor version (e.g. from v0.1.x to v0.2.0). 
I have no intention of ever upgrading the major version (i.e. to v1.~), nor do I intend to ever rigorously maintain this.

I simply find these utilities (or, right now, singular utility) useful, and hope others do as well.

## Installation

Because this is an experimental library that I'm not going to be carefully maintaining, I'm not going to add it to
`cargo`. Instead, simply add the following to your `Cargo.toml` file:

```toml
[dependencies]
cli_testing_utils = { git = "https://github.com/maxblee/cli_testing_utils" , tag = "v0.1.0" }
```
This isn't a stable crate, so I strongly recommend including the tag.

## Development

If you want to develop this or modify it on your own, feel free to. Simply type

```shell
$ git clone https://github.com/maxblee/cli_testing_utils
$ cd cli_testing_utils
```

Since I am not uploading the library to [crates.io](https://crates.io), you will
need to build the documentation yourself to find an API. Once you have `cd`'d
into this crate's directory, simply type:
```shell
$ cargo doc --open
```
to open an HTML API in your browser.