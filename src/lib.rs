//!`clipivot` is a command-line tool for quickly creating pivot tables.
//!
//! If you want to use the program, visit the
//! [Github repo](https://github.com/maxblee/clipivot) for installation
//! and usage instructions. If, on the other hand, you want to contribute to `clipivot`'s
//! development, read on.
//!
//! In particular, I strongly advise you to read the brief bit at the top of the page for the
//! `aggregation` module. That bit should show you how `clipivot` is structured, so you can
//! more knowledgeably explore the tool.
//!
//! # How to help
//! Regardless of your programming experience, you can help make `clipivot` a better tool.
//!
//! ## Requires programming experience
//! - Performance: I've tried to design `clipivot` to be reasonably performant, but I'm sure there
//! are places where performance could be optimized. If you have any suggestions, I'd love to hear them.
//! (Note: I'm aware that there are technically faster algorithms for computing median than the one I
//! wound up with, the [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
//! in Rust's standard library. The reason I chose the `BTreeMap` is that it is well-suited for
//! adding items from a stream and it is more memory efficient than other algorithms I'm aware of.
//! But let me know if you're aware of a way to improve the speed of the median computation
//! while maintaining the best case memory efficiency of `BTreeMap`.)
//! - Coding style: This is my first project in Rust, so I'm sure there are parts of the code
//! that are not idiomatic in Rust or that are poorly structured.
//! - Testing: I think I've included fairly decent testing for this tool, but I'm sure there are places
//! where my testing can improve.
//! - Continuous Integration: Thanks to [two](https://github.com/japaric/trust) [templates](https://github.com/BurntSushi/xsv),
//! I managed to get continuous integration working in Travis CI for two version ins of Linux, one version of OSX,
//! and one version of Windows. However, some versions I tried to deploy failed
//! (they're currently commented out in the .travis.yml file). If anyone wants to help get those working or wants to add support
//! for other environments, I would really appreciate it.
//!
//! ## Doesn't require programming experience
//! - Bugs: If something in this program doesn't work like you think it's supposed to, please let me know.
//! - Error handling: I've tried to make error handling as clear and helpful as possible, so if an error
//!  message you get from `clipivot` confuses you, let me know and I'll do what I can to fix it.
//!
//! In particular, pretty much nothing you run should ever result in what Rust calls a "panic" -- basically an unanticipated,
//! fast exit from a program. Panics look something like:
//!
//! ```sh
//! thread 'main' panicked at 'explicit_panic', src/main.rs:5:1
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! - Documentation: You shouldn't be confused about how to get `clipivot` to work. If you've read the guide
//! on GitHub and the help message and are confused by part of it, please let me know.
//!
//! - Features: I don't have any new features in mind for `clipivot`, but if you do,
//! let me know and I'll consider whether or not I think it makes sense to add the feature.
//!
//! # Development Environment
//! In order to contribute code, first clone the repository to install the source code:
//!
//! ```sh
//! $ git clone https://github.com/maxblee/clipivot
//! ```
//!
//! Then, make changes to the code and/or add/change tests, and then run
//!
//! ```sh
//! $ cargo test
//! ```
//!
//! to run tests.
//! ## Formatting
//! In addition, I use `clippy` to lint code and `rustfmt` to automatically format code.
//!
//!
//! To install them, type
//! ```sh
//! $ rustup update
//! $ rustup component add rustfmt --toolchain stable
//! $ rustup component add clippy --toolchain stable
//! ```
//! And from there, you can run `rustfmt` with
//! ```sh
//! $ cargo fmt --all
//! ```
//! and `clippy` with
//! ```sh
//! $ cargo clippy -- -A clippy::ptr_arg
//! ```
//! **Note that I am ignoring the `clippy::ptr_arg` warning, which raises a warning
//! when you put a `&Vec<T>` into a function call.
//!
//! # Contact me
//! To get in touch with me about `clipivot`, send me an email at <maxbmhlee@gmail.com> or submit an issue on
//! the GitHub page.
pub mod aggfunc;
pub mod aggregation;
pub mod cli_settings;
pub mod errors;
pub mod parsing;
