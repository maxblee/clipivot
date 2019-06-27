//#[test]
//fn test_only_valid_aggfuncs_allowed() {
//    // makes sure you can't accidentally use an aggfunc that is not permitted
//    let cmd = assert_cli::Assert::main_binary();
//    cmd.with_args(&["badcount"])
//        .succeeds();
//}
//
//#[test]
//fn test_valid_aggfunc_succeeds() {
//    // a corollary to `test_only_valid_aggfuncs_allowed` that makes sure
//    // that running the program with valid functions succeeds
//    let cmd = assert_cli::Assert::main_binary();
//    cmd.with_args(&["count"])
//        .succeeds();
//}