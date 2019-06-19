//!`csvpivot` is a command-line tool for quickly creating pivot tables.
//!
//! If you want to use the program, visit the
//! [Github repo](https://github.com/maxblee/csvpivot) for installation
//! and usage instructions. If, on the other hand, you want to contribute to `csvpivot`'s
//! development, read on.
//!
//! This tool is designed for easily and quickly creating pivot tables. Right now, there
//! are a few areas where I'd like to see improvement.
#[macro_use]
extern crate clap;

use clap::App;
use std::process;

pub mod aggregation;
pub mod parsing;
pub mod aggfunc;
pub mod errors;

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