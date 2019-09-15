//! This module tests the standard deviation and mean algorithms in `clipivot`,
//!
//! In order to test the accuracy of the standard deviation and mean algorithms, I've used the
//! Statistical Reference Datasets for univariate summary statistics
//! from NIST, which are designed for comparing means, standard deviations, and
//! autocorrelation coefficients to their certified values.
//!
//! All of the tests assert that the standard deviation algorithms are accurate
//! on all of the NIST datasets to within 9 significant digits. In some extreme cases, the exact
//! performance may be worse than that. For example, one of the test cases in the `aggfunc`
//! module, called `stdev_computation_is_stable` typically performs accurately only to
//! 8 significant digits and sometimes performs accurately to only 7 significant digits.
//! However, that's demonstrating the accuracy of the computation on an unusual edge case
//! and the algorithm still performs fairly well, if imperfectly, on that case.
//!
//! The mean algorithms, meanwhile, are more accurate than that, since they rely solely on
//! Decimal addition and a single Decimal division operation. As these tests show, all the mean
//! algorithms perform accurately on NIST's dataset to at least 12 significant digits;
//! more typically, they perform perfectly accurately. There are some extreme cases in which
//! summation might cause an overflow. Those currently will panic and eventually will throw an error.
use approx::assert_abs_diff_eq;
use std::process::Command;
use std::str;

#[macro_use]
extern crate cli_testing_utils;

type NumericRecord = (String, f64);

fn stddev_epsilon() -> f64 {
    // Returns the epsilon value for all tests in this file
    1e-9
}

fn mean_epsilon() -> f64 {
    1e-12
}

fn get_actual_result(filename: &str, aggfunc: &str) -> f64 {
    // Returns the result from NIST's dataset given the relative file path
    // the match formatting is required to get these tests to work in Travis CI
    let program_name = program_path!();
    let output = Command::new(program_name)
        .args(&[aggfunc, filename, "-v", "0"])
        .output()
        .expect("Process failed to execute")
        .stdout;
    let stroutput = str::from_utf8(&output).unwrap();
    let mut rdr = csv::Reader::from_reader(stroutput.as_bytes());
    let mut iter = rdr.deserialize();
    let item: NumericRecord = iter.next().unwrap().unwrap();
    item.1
}

#[test]
fn test_num_acc1_std() {
    // Makes sure that test_csvs/NumAcc1.csv performs standard deviation calculations accurately
    let result = get_actual_result("test_csvs/NumAcc1.csv", "stddev");
    assert_abs_diff_eq!(result, 1., epsilon = stddev_epsilon());
}

#[test]
fn test_num_acc2_std() {
    let result = get_actual_result("test_csvs/NumAcc2.csv", "stddev");
    assert_abs_diff_eq!(result, 0.1, epsilon = stddev_epsilon());
}

#[test]
fn test_num_acc3_std() {
    let result = get_actual_result("test_csvs/NumAcc3.csv", "stddev");
    assert_abs_diff_eq!(result, 0.1, epsilon = stddev_epsilon());
}

#[test]
fn test_num_acc4_std() {
    let result = get_actual_result("test_csvs/NumAcc4.csv", "stddev");
    assert_abs_diff_eq!(result, 0.1, epsilon = stddev_epsilon());
}

#[test]
fn test_lew_std() {
    let result = get_actual_result("test_csvs/Lew.csv", "stddev");
    assert_abs_diff_eq!(result, 277.332168044316, epsilon = stddev_epsilon());
}

#[test]
fn test_lottery_std() {
    let result = get_actual_result("test_csvs/Lottery.csv", "stddev");
    assert_abs_diff_eq!(result, 291.699727470969, epsilon = stddev_epsilon());
}

#[test]
fn test_mavro_std() {
    let result = get_actual_result("test_csvs/Mavro.csv", "stddev");
    assert_abs_diff_eq!(result, 0.000429123454003053, epsilon = stddev_epsilon());
}

#[test]
fn test_michelso_std() {
    let result = get_actual_result("test_csvs/Michelso.csv", "stddev");
    assert_abs_diff_eq!(result, 0.0790105478190518, epsilon = stddev_epsilon());
}

#[test]
fn test_pi_std() {
    let result = get_actual_result("test_csvs/PiDigits.csv", "stddev");
    assert_abs_diff_eq!(result, 2.86733906028871, epsilon = stddev_epsilon());
}

#[test]
fn test_num_acc1_mean() {
    let result = get_actual_result("test_csvs/NumAcc1.csv", "mean");
    assert_abs_diff_eq!(result, 10000002., epsilon = mean_epsilon());
}

#[test]
fn test_num_acc2_mean() {
    let result = get_actual_result("test_csvs/NumAcc2.csv", "mean");
    assert_abs_diff_eq!(result, 1.2, epsilon = mean_epsilon());
}

#[test]
fn test_num_acc3_mean() {
    let result = get_actual_result("test_csvs/NumAcc3.csv", "mean");
    assert_abs_diff_eq!(result, 1000000.2, epsilon = mean_epsilon());
}

#[test]
fn test_num_acc4_mean() {
    let result = get_actual_result("test_csvs/NumAcc4.csv", "mean");
    assert_abs_diff_eq!(result, 10000000.2, epsilon = mean_epsilon());
}

#[test]
fn test_lottery_mean() {
    let result = get_actual_result("test_csvs/Lottery.csv", "mean");
    assert_abs_diff_eq!(result, 518.958715596330, epsilon = mean_epsilon());
}

#[test]
fn test_mavro_mean() {
    let result = get_actual_result("test_csvs/Mavro.csv", "mean");
    assert_abs_diff_eq!(result, 2.00185600000000, epsilon = mean_epsilon());
}

#[test]
fn test_michelso_mean() {
    let result = get_actual_result("test_csvs/Michelso.csv", "mean");
    assert_abs_diff_eq!(result, 299.852400000000, epsilon = mean_epsilon());
}

#[test]
fn test_pi_mean() {
    let result = get_actual_result("test_csvs/PiDigits.csv", "mean");
    assert_abs_diff_eq!(result, 4.53480000000000, epsilon = mean_epsilon());
}

#[test]
fn test_lew_mean() {
    let result = get_actual_result("test_csvs/Lew.csv", "mean");
    assert_abs_diff_eq!(result, -177.435000000000, epsilon = mean_epsilon());
}
