#![allow(dead_code, unused_doc_comments)]

#[macro_use]
extern crate clap;

use clap::App;
use std::process;

mod log_query;
mod aggregation;

fn main() {
    let yaml_file = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml_file)
        .version(crate_version!())
        .author(crate_authors!());
    let config = match aggregation::CliConfig::from_app(app) {
        Ok(trial_config) => trial_config,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    if let Err(err) = aggregation::run(config) {
        println!("{}", err);
        process::exit(1);
    }
}
