use assert_cmd::Command;
use std::process::Output;
/// This module holds most of the integration tests (basically everything but numerical accuracy tests)
use std::str;

#[macro_use]
mod common;

fn setup_cmd(query: &[&str]) -> Output {
    let program_name = program_path!();
    Command::new(program_name)
        .args(query)
        .output()
        .expect("Processed failed to execute")
}

fn setup_results(query: &[&str]) -> Vec<Vec<String>> {
    let mut results = vec![];
    let output = setup_cmd(query).stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    for result in rdr.deserialize() {
        let record: Vec<String> = result.unwrap();
        results.push(record);
    }
    results
}

fn setup_sorting_columns(query: &[&str]) -> Vec<String> {
    let output = setup_cmd(query).stdout;
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
    args.push("--index-cols");
    let mult_status = setup_cmd(&args).status;
    assert!(!mult_status.success());
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
    let index_order = setup_results(&args);
    let index_expected = vec![
        vec!["2017".to_string(), "2".to_string()],
        vec!["2018".to_string(), "2".to_string()],
        vec!["2016".to_string(), "1".to_string()],
        vec!["2019".to_string(), "1".to_string()],
    ];
    assert_eq!(index_order, index_expected);
    args.push("--asc-rows");
    let sorted_asc = setup_results(&args);
    let mut sorting_expected = vec![
        vec!["2016".to_string(), "1".to_string()],
        vec!["2017".to_string(), "2".to_string()],
        vec!["2018".to_string(), "2".to_string()],
        vec!["2019".to_string(), "1".to_string()],
    ];
    assert_eq!(sorted_asc, sorting_expected);
    args[6] = "--desc-rows";
    let sorted_desc = setup_results(&args);
    sorting_expected.reverse();
    assert_eq!(sorted_desc, sorting_expected);
    args.push("--asc-rows");
    let asc_and_desc_status = setup_cmd(&args).status;
    assert!(!asc_and_desc_status.success());
}

#[test]
fn test_flag_ignores_empty_vals() {
    let query = ["count", "test_csvs/empty_count.csv", "-v", "2", "-e"];
    let output = setup_cmd(&query).stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    let mut iter = rdr.deserialize();
    let item: (String, usize) = iter.next().unwrap().unwrap();
    assert_eq!(item.1, 1);
}
#[test]
fn test_wo_e_flag_parses_empty_vals() {
    let query = ["count", "test_csvs/empty_count.csv", "-v", "2"];
    let output = setup_cmd(&query).stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    let mut iter = rdr.deserialize();
    let item: (String, usize) = iter.next().unwrap().unwrap();
    assert_eq!(item.1, 2);
}

#[test]
fn test_noheader() {
    let mut query = vec!["count", "test_csvs/one_liner.csv", "-v", "0"];
    let output = setup_cmd(&query);
    assert!(!output.status.success());
    query.push("--no-header");
    let new_output = setup_results(&query);
    assert_eq!(new_output, vec![vec!["total".to_string(), "1".to_string()]]);
}

#[test]
fn test_tab_delimiter() {
    let file_query = vec!["count", "test_csvs/tab_tsv.tsv", "-v", "0"];
    let output = setup_cmd(&file_query);
    assert!(output.status.success());
    assert_eq!(
        setup_results(&file_query),
        vec![vec!["total".to_string(), "2".to_string()]]
    );
    let stdin_contents = "foo, bar	bar	baz
    aaa	bbb	ccc
    1	2	3";
    let _cmd = Command::new(program_path!())
        .args(vec!["count", "-v", "0"])
        .write_stdin(stdin_contents)
        .assert()
        // should fail because of the comma on the header
        .failure();
    let _cmd_with_t = Command::new(program_path!())
        .args(vec!["count", "-v", "0", "-t"])
        .write_stdin(stdin_contents)
        .assert()
        .success();
    let _cmd_with_delim = Command::new(program_path!())
        .args(vec!["count", "-v", "0", "-d", "\t"])
        .write_stdin(stdin_contents)
        .assert()
        .success();
}

#[test]
fn test_custom_delim() {
    let stdin_contents = "1$2,a$3
    a$b$c";
    let _cmd_without_delim = Command::new(program_path!())
        .args(vec!["count", "-v", "0"])
        .write_stdin(stdin_contents)
        .assert()
        .failure();
    let _cmd_with_delim = Command::new(program_path!())
        .args(vec!["count", "-v", "0", "-d", "$"])
        .write_stdin(stdin_contents)
        .assert()
        .success();
}
