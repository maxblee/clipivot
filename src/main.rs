use std::process;

use clipivot::cli;
use clipivot::parsing;
fn main() {
    if let Some(date_format) = cli::CLI_ARGS.value_of("format") {
        parsing::set_date_format(date_format.to_string());
    }

    if let Err(err) = cli::run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}
