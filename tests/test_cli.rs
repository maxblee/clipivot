use std::process::Command;
/// This module holds most of the integration tests (basically everything but numerical accuracy tests)
use std::str;

#[macro_use]
extern crate cli_testing_utils;

type SimpleCount = (String, usize);
type StringRec = Vec<String>;

fn setup_sorting_tests(query: &[&str]) -> Vec<StringRec> {
    let mut results = vec![];
    let program_name = program_path!();
    let output = Command::new(program_name)
        .args(query)
        .output()
        .expect("Process failed to execute")
        .stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    for result in rdr.deserialize() {
        let record: StringRec = result.unwrap();
        results.push(record);
    }
    results
}

fn setup_sorting_columns(query: &[&str]) -> Vec<String> {
    let program_name = program_path!();
    let output = Command::new(program_name)
        .args(query)
        .output()
        .expect("Process failed to execute")
        .stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    // skip so we ignore row column
    rdr.headers()
        .unwrap()
        .iter()
        .skip(1)
        .map(String::from)
        .collect()
}

#[test]
fn test_column_sorting() {
    let mut args = vec![
        "count",
        "test_csvs/sorting_csv.csv",
        "-v",
        "0",
        "-c",
        "year",
    ];
    let asc_order = setup_sorting_columns(&args);
    let mut sorting_expected = vec![
        "2016".to_string(),
        "2017".to_string(),
        "2018".to_string(),
        "2019".to_string(),
    ];
    assert_eq!(asc_order, sorting_expected);
    args.push("--index-cols");
    let index_expected = vec![
        "2017".to_string(),
        "2018".to_string(),
        "2016".to_string(),
        "2019".to_string(),
    ];
    let index_order = setup_sorting_columns(&args);
    assert_eq!(index_order, index_expected);
    sorting_expected.reverse();
    args[6] = "--desc-cols";
    let desc_order = setup_sorting_columns(&args);
    assert_eq!(desc_order, sorting_expected);
}

#[test]
fn test_index_sorting_works() {
    let mut args = vec![
        "count",
        "test_csvs/sorting_csv.csv",
        "-v",
        "0",
        "-r",
        "year",
    ];
    let index_order = setup_sorting_tests(&args);
    let index_expected = vec![
        vec!["2017".to_string(), "2".to_string()],
        vec!["2018".to_string(), "2".to_string()],
        vec!["2016".to_string(), "1".to_string()],
        vec!["2019".to_string(), "1".to_string()],
    ];
    assert_eq!(index_order, index_expected);
    args.push("--asc-rows");
    let sorted_asc = setup_sorting_tests(&args);
    let mut sorting_expected = vec![
        vec!["2016".to_string(), "1".to_string()],
        vec!["2017".to_string(), "2".to_string()],
        vec!["2018".to_string(), "2".to_string()],
        vec!["2019".to_string(), "1".to_string()],
    ];
    assert_eq!(sorted_asc, sorting_expected);
    args[6] = "--desc-rows";
    let sorted_desc = setup_sorting_tests(&args);
    sorting_expected.reverse();
    assert_eq!(sorted_desc, sorting_expected);
}

#[test]
fn test_flag_ignores_empty_vals() {
    let output = Command::new(program_path!())
        .args(&["count", "test_csvs/empty_count.csv", "-v", "2", "-e"])
        .output()
        .expect("Process failed to execute")
        .stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    let mut iter = rdr.deserialize();
    let item: SimpleCount = iter.next().unwrap().unwrap();
    assert_eq!(item.1, 1);
}
#[test]
fn test_wo_e_flag_parses_empty_vals() {
    let output = Command::new(program_path!())
        .args(&["count", "test_csvs/empty_count.csv", "-v", "2"])
        .output()
        .expect("Process failed to execute")
        .stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    let mut iter = rdr.deserialize();
    let item: SimpleCount = iter.next().unwrap().unwrap();
    assert_eq!(item.1, 2);
}
