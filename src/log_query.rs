extern crate chrono;

use std::io;
use std::env;
use std::io::prelude::*;
use std::fs::OpenOptions;
use chrono::prelude::Local;


fn get_time() -> String {
    // returns the current datetime in YYYY-MM-DD HH:MM:SS 24-hour format
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn parse_message() -> String {
    // reads message from standard input and returns it
    let mut msg = String::new();
    println!("Describe what this query does, why you ran it, and what it shows:");
    io::stdin().read_line(&mut msg)
        .expect("Please enter a message");
    msg
}

fn parse_query() -> String {
    // returns the String you entered into the command line
    let mut query_txt : Vec<String> = Vec::new();
    let query = env::args();
    for arg in query {
        query_txt.push(arg);
    }
    query_txt.join(" ")
}

pub fn update_diary(filename: &str) {
    // appends or creates data diary with new record. Adds creation time if creating
    let str_time = get_time();
    let log_info = format!("{}\n\t{}\tQuery: {}", str_time, parse_message(), parse_query());
    let mut fp = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let mut reader = io::BufReader::new(&fp);
    // https://jonalmeida.com/posts/2015/03/03/rust-new-io/
    let mut buf_str = String::new();
    reader.read_line(&mut buf_str)
        .expect("Ran into trouble reading the file. Is the file valid UTF-8?");
    if buf_str.is_empty() {
        if let Err(e) = writeln!(fp, "Data diary {} was created at {}", filename, str_time) {
            eprintln!("Couldn't write to file `{}`: {}", filename, e);
        };
    }
    // https://stackoverflow.com/questions/30684624/what-is-the-best-variant-for-appending-a-new-line-in-a-text-file
    if let Err(e) = writeln!(fp, "{}", log_info) {
        eprintln!("Couldn't write to file `{}`: {}", filename, e);
    }
}