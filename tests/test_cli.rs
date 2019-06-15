#[test]
fn test_only_valid_aggfuncs_allowed() {
    // makes sure you can't accidentally use an aggfunc that is not permitted
    let cmd = assert_cli::Assert::main_binary();
    cmd.with_args(&["badcount"])
        .fails();
}