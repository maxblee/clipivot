/// Finds the path to your program, whether you are running locally
/// or using Travis-CI. (**Note: Does not support other CI schemes.**)
///
/// This macro exists so you can easily run a command-line program
/// in an integration test, whether you are running the program
/// locally (in which case the path is just "target/debug/YOUR_PROGRAM_NAME")
/// or using Travis-CI.
///
/// Here's an example of how you might use it:
/// 
/// ```ignore
/// use std::process::Command;
/// let cmd = Command::new(program_path!());
/// ```
///
/// (The structure of this was heavily inspired from the macros in
/// [Clap](https://github.com/clap-rs/clap/blob/master/src/macros.rs#L147))
/// and from
/// [this helpful guide to implementing macros](https://medium.com/@phoomparin/a-beginners-guide-to-rust-macros-5c75594498f1).
#[cfg(not(feature = "no_cargo"))]
#[macro_export]
macro_rules! program_path {
    () => {{
        let package_name = env!("CARGO_PKG_NAME");
        &match option_env!("TARGET") {
            Some(target_loc) => format!("target/{}/debug/{}", target_loc, package_name),
            None => format!("target/debug/{}", package_name)
        }
    }};
}

#[cfg(test)]
mod test {
    #[test]
    fn no_target_yields_local_result() {
        // If there isn't a TARGET env variable, returns target/debug/PKG_NAME
        assert_eq!(program_path!(), "target/debug/cli_testing_utils");
        // Note: the option_env! macro checks the environment variable *at compile time*
        // so I can't directly mock that with an env::set_var call
    }
}