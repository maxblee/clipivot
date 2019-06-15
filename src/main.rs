//! `csvpivot` is a tool for creating pivot tables from the command-line.
//! If that sounds like something you'd use, check out
//! [the GitHub repo](https://github.com/maxblee/csvpivot) for a usage guide.
//! If you want to contribute to improving `csvpivot` (or to fixing bugs),
//! I'd recommend you click through the `aggregations` module and read through
//! some of the source code for a sense of how the program is structured.
//!
//! That should also give you a sense of some of the things I'd like to see addressed
//! in future versions. I've also included a list of the changes I'd like to see,
//! ranked by how important I think they are:
//! 1. More aggregation methods: Right now, I only have support for aggregating by count.
//! But I'd like this program to become a lot more extensive.
//! 2. Text parsing: Currently, the program only parsing text as just that: text.
//! But in order to deal with sums, minimums, etc. I need to be able to handle numbers.
//! I'd also *eventually* like to handle dates in a reasonable manner.
//! 3. Better error messages: Right now, my custom error messages are pretty vague.
//! If you have ideas of how I could change that (and most importantly, an understanding
//! of how to implement those changes), please let me know).
//! 4. Code re-use: Right now, the code for reading from standard input and for reading
//! from a file is basically copied into separate functions. I'd like to refactor the code
//! to limit the amount of code re-use.
//! 5. Allowing for aggregating solely based on a column/row: Right now, you have to aggregate
//! based on column AND row AND select a value. But I'd like to make that optional,
//! replacing the null vector with "total."
//! 6. Handling non-UTF-8 data: Right now, ASCII data that isn't UTF-8 will
//! return an error. That should eventually be changed
//! 7. Right now, if you aggregate based on multiple rows or columns, the name of the values
//! are separated by a "$." separator. I suspect there is a better solution, but I don't know what it is.
//! 8. Additional configuration options, for instance allowing for files that aren't comma-separated
//! to be properly handled.
//! 9. Performance considerations: I've built this to be *reasonably* performant, in the sense
//! that I've tried to use online algorithms that can be processed with one read through the original
//! data and with one read through the aggregated data when possible. But I'm sure there are a number
//! of places where performance could be improved (especially, I suspect, by limiting the degree
//! to which I've copied, rather than referenced/borrowed, strings.
//!
//! If you want to work on addressing any of these issues or have ideas of your own
//! you'd like to see implemented here, contact me at maxbmhlee@gmail.com.

#![allow(unused_doc_comments)]

#[macro_use]
extern crate clap;

use clap::App;
use std::process;

mod log_query;
mod aggregation;

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
