use std::panic;
use std::process::Command;
/// This module holds most of the integration tests (basically everything but numerical accuracy tests)
use std::str;

const PROGRAM_NAME: &str = "./target/debug/csvpivot";
type SimpleCount = (String, usize);

#[test]
fn test_flag_ignores_empty_vals() {
    let output = Command::new(PROGRAM_NAME)
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
    let output = Command::new(PROGRAM_NAME)
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
