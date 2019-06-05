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
    let mut query = String::new();
    for arg in env::args() {
        query.push_str(&arg);
        query.push(' ');
    }
    query
}

pub fn update_diary(filename: &str) {
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
    let mut buf_str = String::new();
    reader.read_line(&mut buf_str);
    if buf_str.is_empty() {
        writeln!(fp, "Data diary {} was created at {}", filename, str_time);
    }
    if let Err(e) = writeln!(fp, "{}", log_info) {
        eprintln!("Couldn't write to file `{}`: {}", filename, e);
    }
}