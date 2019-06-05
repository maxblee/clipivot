#[macro_use]
extern crate clap;

use clap::App;

mod log_query;

fn main() {
    let yaml_file = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml_file)
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    //handle the filename (required) argument
    if let Some(fname) = matches.value_of("FILENAME") {
//        println!("Do something with {}", fname);
    }

    if let Some(diary_file) = matches.value_of("log") {
//        println!("{}\n\t{}\tQuery: {}", get_time(), parse_message(), parse_query());
        log_query::update_diary(diary_file);
    }
}
