//!`csvpivot` is a command-line tool for quickly creating pivot tables.
//!
//! If you want to use the program, visit the
//! [Github repo](https://github.com/maxblee/csvpivot) for installation
//! and usage instructions. If, on the other hand, you want to contribute to `csvpivot`'s
//! development, read on.
//!
//! # How to help
//! Regardless of your programming experience, you can help make `csvpivot` a better tool.
//! 
//! ## Requires programming experience
//! - Performance: I've tried to design `csvpivot` to be reasonably performant, but I'm sure there
//! are places where performance could be optimized. If you have any suggestions, I'd love to hear them.
//! (Note: I'm aware that there are technically faster algorithms for computing median than the one I
//! wound up with, the [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
//! in Rust's standard library. The reason I chose the `BTreeMap` is that it is well-suited for
//! adding items from a stream and it is more memory efficient than other algorithms I'm aware of.
//! But let me know if you're aware of a way to improve the speed of the median computation
//! while maintaining the best case memory efficiency of `BTreeMap`.)
//! - Coding style: This is my first project in Rust, so I'm sure there are parts of the code 
//! that are not idiomatic in Rust or that are poorly structured.
//! - Testing: I've tried to have robust testing for this tool, but text data and (barely existent)
//! CSV standards are both full of edge cases. So if there are any additional tests you think the program
//! needs, let me know or make a pull request.
//!
//! ## Doesn't require programming experience
//! - Bugs: If something in this program doesn't work like you think it's supposed to, please let me know.
//! - Error handling: I've tried to make error handling as clear and helpful as possible, so if an error
//!  message you get from `csvpivot` confuses you, let me know and I'll do what I can to fix it.
//!
//! In particular, pretty much nothing you run should ever result in what Rust calls a "panic" -- basically an unanticipated,
//! fast exit from a program. Panics look something like:
//!
//! ```sh
//! thread 'main' panicked at 'explicit_panic', src/main.rs:5:1
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//! 
//! The only exceptions I can think of are that summation and other mathematical operations can techinically
//! result in overflows, and that some of the algorithms can potentially cause you to run out of memory.
//! But both of those examples should be exceptionally rare (even when dealing with datasets larger than your RAM),
//! so if you ever run into a panic, please send me a bit of information about the query you ran so I can fix this.
//!
//! - Documentation: You shouldn't be confused about how to get `csvpivot` to work. If you've read the guide
//! on GitHub and the help message and are confused by part of it, please let me know.
//!
//! - Features: I'm not particularly envisioning any new features for `csvpivot`, but if you have an idea for a feature,
//! let me know and I'll consider whether or not I think it makes sense to add the feature.
//!
//! # Development Environment
//! In order to contribute, first clone the repository to install the source code:
//!
//! ```sh
//! $ git clone https://github.com/maxblee/csvpivot
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
//! Or `clippy` with
//! ```sh
//! $ cargo clippy
//! ```
//! # Contact me
//! To get in touch with me about `csvpivot`, send me an email at <maxbmhlee@gmail.com> or submit an issue on
//! the GitHub page.

#[macro_use]
extern crate clap;

use clap::App;
use std::process;

pub mod aggfunc;
pub mod aggregation;
pub mod errors;
pub mod parsing;


fn main() {
    let yaml_file = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml_file)
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    if let Err(err) = aggregation::run(matches) {
        println!("{}", err);
        process::exit(1);
    }
}
