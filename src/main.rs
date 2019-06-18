#[macro_use]
extern crate clap;

use clap::App;
use std::process;

pub mod aggregation;
pub mod fun;

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